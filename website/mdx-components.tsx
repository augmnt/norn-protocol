import type { MDXComponents } from "mdx/types";
import Link from "next/link";

export function useMDXComponents(components: MDXComponents): MDXComponents {
  return {
    h1: ({ children, ...props }) => (
      <h1
        className="scroll-m-20 text-3xl font-bold tracking-tight mb-6"
        {...props}
      >
        {children}
      </h1>
    ),
    h2: ({ children, ...props }) => (
      <h2
        className="scroll-m-20 text-xl font-semibold tracking-tight mt-10 mb-4 border-b pb-2"
        {...props}
      >
        {children}
      </h2>
    ),
    h3: ({ children, ...props }) => (
      <h3
        className="scroll-m-20 text-lg font-semibold tracking-tight mt-8 mb-3"
        {...props}
      >
        {children}
      </h3>
    ),
    h4: ({ children, ...props }) => (
      <h4
        className="scroll-m-20 text-base font-semibold tracking-tight mt-6 mb-2"
        {...props}
      >
        {children}
      </h4>
    ),
    p: ({ children, ...props }) => (
      <p className="leading-7 [&:not(:first-child)]:mt-4 text-muted-foreground" {...props}>
        {children}
      </p>
    ),
    a: ({ href, children, ...props }) => {
      const isExternal = href?.startsWith("http");
      if (isExternal) {
        return (
          <a
            href={href}
            target="_blank"
            rel="noopener noreferrer"
            className="text-norn underline underline-offset-4 hover:text-norn/80"
            {...props}
          >
            {children}
          </a>
        );
      }
      return (
        <Link
          href={href ?? ""}
          className="text-norn underline underline-offset-4 hover:text-norn/80"
          {...props}
        >
          {children}
        </Link>
      );
    },
    ul: ({ children, ...props }) => (
      <ul className="my-4 ml-6 list-disc [&>li]:mt-2 text-muted-foreground" {...props}>
        {children}
      </ul>
    ),
    ol: ({ children, ...props }) => (
      <ol className="my-4 ml-6 list-decimal [&>li]:mt-2 text-muted-foreground" {...props}>
        {children}
      </ol>
    ),
    li: ({ children, ...props }) => (
      <li className="leading-7" {...props}>
        {children}
      </li>
    ),
    blockquote: ({ children, ...props }) => (
      <blockquote
        className="mt-4 border-l-2 border-norn/30 pl-4 italic text-muted-foreground"
        {...props}
      >
        {children}
      </blockquote>
    ),
    code: ({ children, ...props }) => (
      <code
        className="relative rounded bg-muted px-[0.4rem] py-[0.2rem] font-mono text-sm"
        {...props}
      >
        {children}
      </code>
    ),
    pre: ({ children, ...props }) => (
      <pre
        className="my-4 overflow-x-auto rounded-lg border bg-secondary/50 p-4 text-sm"
        {...props}
      >
        {children}
      </pre>
    ),
    table: ({ children, ...props }) => (
      <div className="my-6 w-full overflow-x-auto">
        <table className="w-full text-sm" {...props}>
          {children}
        </table>
      </div>
    ),
    thead: ({ children, ...props }) => (
      <thead className="border-b bg-muted/30" {...props}>
        {children}
      </thead>
    ),
    th: ({ children, ...props }) => (
      <th
        className="px-4 py-3 text-left font-medium text-muted-foreground"
        {...props}
      >
        {children}
      </th>
    ),
    td: ({ children, ...props }) => (
      <td className="px-4 py-3 border-b" {...props}>
        {children}
      </td>
    ),
    tr: ({ children, ...props }) => (
      <tr className="border-b last:border-0" {...props}>
        {children}
      </tr>
    ),
    hr: (props) => <hr className="my-8 border-t" {...props} />,
    strong: ({ children, ...props }) => (
      <strong className="font-semibold text-foreground" {...props}>
        {children}
      </strong>
    ),
    ...components,
  };
}
