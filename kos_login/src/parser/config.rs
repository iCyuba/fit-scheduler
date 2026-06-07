use bytes::Bytes;
use futures::{StreamExt, stream::Stream};
use lol_html::{HtmlRewriter, Settings, errors::RewritingError, text};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigParseError<T> {
    #[error("$Config variable is missing from the login page")]
    MissingConfig,

    #[error(transparent)]
    ByteStreamError(T),

    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),

    #[error(transparent)]
    RewritingError(#[from] RewritingError),
}

#[derive(Debug, Error)]
#[error("Done!")]
struct DoneError;

const PREFIX: &str = "//<![CDATA[\n$Config=";
const SUFFIX: &str = ";\n//]]>";

pub async fn parse_config<T, S: Stream<Item = Result<Bytes, T>> + Unpin>(
    stream: &mut S,
) -> Result<Value, ConfigParseError<T>> {
    let mut config = String::new();
    let mut skip = false;

    let settings = Settings::new().append_element_content_handler(text!(
        "script[type=\"text/javascript\"]",
        |t| {
            if !skip {
                let len = config.len();
                config.push_str(t.as_str());

                if len < PREFIX.len() && config.len() > PREFIX.len() {
                    skip = !config.starts_with(PREFIX);
                }
            }

            if t.last_in_text_node() {
                if config.starts_with(PREFIX) && config.ends_with(SUFFIX) {
                    Err(DoneError)?
                } else {
                    config.clear();
                    skip = false;
                }
            }

            Ok(())
        }
    ));

    let mut rewriter = HtmlRewriter::new(settings, |_: &[u8]| {});

    let mut done = false;

    while let Some(bytes) = stream.next().await {
        let bytes = bytes.map_err(|e| ConfigParseError::ByteStreamError(e))?;
        let result = rewriter.write(&bytes);

        if let Err(RewritingError::ContentHandlerError(err)) = result {
            // Return early when the $Config variable is found
            // This returns an special error type, which is also the only possible error

            if err.is::<DoneError>() {
                done = true;
                break;
            } else {
                unreachable!("DoneError is the only error that can happen.")
            }
        } else {
            result?;
        }
    }

    if !done {
        return Err(ConfigParseError::MissingConfig);
    }

    // Drop is required here, because the done "error" has to be thrown.
    // Calling end would panic
    drop(rewriter);

    let config = &config[PREFIX.len()..(config.len() - SUFFIX.len())];
    let config = serde_json::from_str::<Value>(config)?;

    Ok(config)
}

#[cfg(test)]
mod test {
    use bytes::Bytes;
    use const_format::formatc;
    use futures::stream;
    use serde_json::json;
    use std::pin::pin;

    use super::{ConfigParseError, PREFIX, SUFFIX, parse_config};

    const SCRIPT: &str = formatc!("<script type=text/javascript>{PREFIX}");

    #[tokio::test]
    async fn test_example() {
        const EXAMPLE_JSON: &str = "{\"key\": 0}";
        const EXAMPLE: &str = formatc!("<html>{SCRIPT}{EXAMPLE_JSON}{SUFFIX}</script></html>");
        const EXAMPLE_HTML: Bytes = Bytes::from_static(EXAMPLE.as_bytes());

        let mut stream = pin!(stream::once(async { Ok::<_, ()>(EXAMPLE_HTML) }));

        let config = parse_config(&mut stream).await.unwrap();

        assert_eq!(config, json!({"key": 0}))
    }

    #[tokio::test]
    async fn test_example2() {
        const EXAMPLE: &str = formatc!(
            "<html><script type=text/javascript>testtesttesttest</script>{SCRIPT}testtesttesttest</script>{SCRIPT}[]{SUFFIX}</script>{SCRIPT}{PREFIX}testtesttesttest</script></html>"
        );
        const EXAMPLE_HTML: Bytes = Bytes::from_static(EXAMPLE.as_bytes());

        let mut stream = pin!(stream::once(async { Ok::<_, ()>(EXAMPLE_HTML) }));

        let config = parse_config(&mut stream).await.unwrap();

        assert_eq!(config, json!([]));
    }

    #[tokio::test]
    async fn test_empty() {
        const EXAMPLE: Bytes = Bytes::from_static(b"<html></html>");

        let mut stream = pin!(stream::once(async { Ok::<_, ()>(EXAMPLE) }));

        let config = parse_config(&mut stream)
            .await
            .expect_err("No config in HTML");

        assert!(matches!(config, ConfigParseError::MissingConfig));
    }

    #[tokio::test]
    async fn test_invalid_html() {
        const EXAMPLE: Bytes = Bytes::from_static(b"</>");

        let mut stream = pin!(stream::once(async { Ok::<_, ()>(EXAMPLE) }));

        let config = parse_config(&mut stream).await;

        assert!(matches!(config, Err(_)))
    }
}
