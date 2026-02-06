use std::collections::HashMap;
use std::time::Instant;

/// A token bucket for rate limiting.
///
/// Tokens are refilled over time at a constant rate, up to a maximum capacity.
/// Consumers attempt to take tokens; if insufficient tokens are available, the
/// request is denied.
pub struct TokenBucket {
    capacity: u64,
    tokens: u64,
    refill_rate: u64, // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket with the given capacity and refill rate (tokens/sec).
    /// Starts full.
    pub fn new(capacity: u64, refill_rate: u64) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Create a token bucket with a specific starting instant (for testing).
    #[cfg(test)]
    fn new_at(capacity: u64, refill_rate: u64, now: Instant) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
            last_refill: now,
        }
    }

    /// Refill tokens based on elapsed time since the last refill.
    pub fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let new_tokens = elapsed.as_secs() * self.refill_rate;

        if new_tokens > 0 {
            self.tokens = (self.tokens + new_tokens).min(self.capacity);
            self.last_refill = now;
        }
    }

    /// Try to consume the given number of tokens. Returns true if successful.
    /// Refills first based on elapsed time.
    pub fn try_consume(&mut self, tokens: u64) -> bool {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    /// Get the number of currently available tokens (without refilling).
    pub fn available(&self) -> u64 {
        self.tokens
    }
}

/// Rate limiter that enforces both per-peer and global rate limits.
pub struct RateLimiter {
    per_peer: HashMap<String, TokenBucket>,
    global: TokenBucket,
    peer_capacity: u64,
    peer_refill_rate: u64,
}

impl RateLimiter {
    /// Create a new rate limiter.
    ///
    /// - `peer_capacity`: max tokens per peer bucket
    /// - `peer_refill_rate`: tokens per second per peer
    /// - `global_capacity`: max tokens for the global bucket
    /// - `global_refill_rate`: tokens per second for the global bucket
    pub fn new(
        peer_capacity: u64,
        peer_refill_rate: u64,
        global_capacity: u64,
        global_refill_rate: u64,
    ) -> Self {
        Self {
            per_peer: HashMap::new(),
            global: TokenBucket::new(global_capacity, global_refill_rate),
            peer_capacity,
            peer_refill_rate,
        }
    }

    /// Check if a request from a given peer is within rate limits.
    ///
    /// Returns true if both the peer-level and global-level budgets allow
    /// the requested number of tokens. Consumes from both if allowed.
    pub fn check_rate_limit(&mut self, peer_id: &str, tokens: u64) -> bool {
        // Get or create the peer bucket.
        let peer_bucket = self
            .per_peer
            .entry(peer_id.to_string())
            .or_insert_with(|| TokenBucket::new(self.peer_capacity, self.peer_refill_rate));

        // Refill both buckets first to check.
        peer_bucket.refill();
        self.global.refill();

        // Check both have enough tokens before consuming.
        if peer_bucket.available() >= tokens && self.global.available() >= tokens {
            // Consume from both (we already refilled, so call internal consume).
            peer_bucket.tokens -= tokens;
            self.global.tokens -= tokens;
            true
        } else {
            false
        }
    }

    /// Remove a peer's bucket (e.g., when they disconnect).
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.per_peer.remove(peer_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_token_bucket_initial_capacity() {
        let bucket = TokenBucket::new(100, 10);
        assert_eq!(bucket.available(), 100);
    }

    #[test]
    fn test_token_bucket_consume() {
        let mut bucket = TokenBucket::new(100, 10);
        assert!(bucket.try_consume(50));
        assert_eq!(bucket.available(), 50);
        assert!(bucket.try_consume(50));
        assert_eq!(bucket.available(), 0);
    }

    #[test]
    fn test_token_bucket_exceed() {
        let mut bucket = TokenBucket::new(10, 1);
        assert!(bucket.try_consume(10));
        assert!(!bucket.try_consume(1));
        assert_eq!(bucket.available(), 0);
    }

    #[test]
    fn test_token_bucket_refill() {
        let now = Instant::now();
        let mut bucket = TokenBucket::new_at(100, 10, now);

        // Consume all tokens.
        assert!(bucket.try_consume(100));
        assert_eq!(bucket.available(), 0);

        // Simulate time passing by adjusting the last_refill back.
        bucket.last_refill = now - Duration::from_secs(5);
        bucket.refill();

        // Should have gained 5 * 10 = 50 tokens.
        assert_eq!(bucket.available(), 50);
    }

    #[test]
    fn test_token_bucket_refill_capped_at_capacity() {
        let now = Instant::now();
        let mut bucket = TokenBucket::new_at(100, 10, now);

        // Consume half.
        assert!(bucket.try_consume(50));
        assert_eq!(bucket.available(), 50);

        // Wait a long time — should cap at capacity.
        bucket.last_refill = now - Duration::from_secs(1000);
        bucket.refill();
        assert_eq!(bucket.available(), 100);
    }

    #[test]
    fn test_rate_limiter_per_peer() {
        let mut limiter = RateLimiter::new(10, 1, 1000, 1000);

        // Peer A can consume up to 10.
        for _ in 0..10 {
            assert!(limiter.check_rate_limit("peer_a", 1));
        }
        // Peer A is now exhausted.
        assert!(!limiter.check_rate_limit("peer_a", 1));

        // Peer B is independent and can still consume.
        assert!(limiter.check_rate_limit("peer_b", 1));
    }

    #[test]
    fn test_rate_limiter_global_limit() {
        // Global bucket of 5, per-peer of 100 (generous).
        let mut limiter = RateLimiter::new(100, 100, 5, 0);

        // Different peers share the global pool.
        assert!(limiter.check_rate_limit("peer_a", 2));
        assert!(limiter.check_rate_limit("peer_b", 2));
        assert!(limiter.check_rate_limit("peer_c", 1));

        // Global is now at 0.
        assert!(!limiter.check_rate_limit("peer_d", 1));
    }

    #[test]
    fn test_rate_limiter_remove_peer() {
        let mut limiter = RateLimiter::new(5, 0, 1000, 1000);

        // Exhaust peer_a's bucket.
        for _ in 0..5 {
            assert!(limiter.check_rate_limit("peer_a", 1));
        }
        assert!(!limiter.check_rate_limit("peer_a", 1));

        // Remove peer_a — they get a fresh bucket next time.
        limiter.remove_peer("peer_a");
        assert!(limiter.check_rate_limit("peer_a", 1));
    }
}
