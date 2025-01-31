import { Fragment } from 'react'
import Highlight, { defaultProps } from 'prism-react-renderer'
import { useTheme } from 'next-themes'
import lightTheme from 'prism-react-renderer/themes/github'


export function Fence({ children, language }) {
  const { resolvedTheme } = useTheme()
  return (
    <Highlight
      {...defaultProps}
      code={children.trimEnd()}
      language={language}
      theme={resolvedTheme !== 'dark' ? lightTheme: undefined }
    >
      {({ className, style, tokens, getTokenProps }) => (
        <pre className={className} style={style}>
          <code>
            {tokens.map((line, index) => (
              <Fragment key={index}>
                {line.map((token, index) => (
                  <span key={index} {...getTokenProps({ token })} />
                ))}
                {'\n'}
              </Fragment>
            ))}
          </code>
        </pre>
      )}
    </Highlight>
  )
}
