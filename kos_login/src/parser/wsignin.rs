use std::borrow::Cow;

use bytes::Bytes;
use futures::{Stream, StreamExt};
use html_escape::decode_html_entities;
use lol_html::{HtmlRewriter, Settings, element, errors::RewritingError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct WSignInForm {
    #[serde(rename = "wresult")]
    pub result: String,

    #[serde(rename = "wctx")]
    pub ctx: String,
}

#[derive(Debug, Error)]
pub enum WSignInParseError<T> {
    #[error("wresult or wctx is missing")]
    MissingData,

    #[error(transparent)]
    ByteStreamError(T),

    #[error(transparent)]
    RewritingError(#[from] RewritingError),
}

pub async fn parse_wsignin<T, S: Stream<Item = Result<Bytes, T>> + Unpin>(
    stream: &mut S,
) -> Result<WSignInForm, WSignInParseError<T>> {
    let mut result = None;
    let mut ctx = None;

    let settings = Settings::new()
        .append_element_content_handler(element!("input[name=\"wresult\"]", |el| {
            result = el.get_attribute("value");

            Ok(())
        }))
        .append_element_content_handler(element!("input[name=\"wctx\"]", |el| {
            ctx = el.get_attribute("value");

            Ok(())
        }));

    let mut rewriter = HtmlRewriter::new(settings, |_: &[u8]| {});

    while let Some(bytes) = stream.next().await {
        let bytes = bytes.map_err(|e| WSignInParseError::ByteStreamError(e))?;
        rewriter.write(&bytes)?;
    }

    rewriter.end()?;

    let (result, ctx) = (
        result.ok_or(WSignInParseError::MissingData)?,
        ctx.ok_or(WSignInParseError::MissingData)?,
    );

    let form = WSignInForm {
        result: match decode_html_entities(&result) {
            Cow::Borrowed(_) => result,
            Cow::Owned(str) => str,
        },
        ctx: match decode_html_entities(&ctx) {
            Cow::Borrowed(_) => ctx,
            Cow::Owned(str) => str,
        },
    };

    Ok(form)
}

#[cfg(test)]
mod test {
    use bytes::Bytes;
    use futures::stream;
    use std::pin::pin;

    use super::{WSignInParseError, parse_wsignin};

    #[tokio::test]
    async fn test_example() {
        const EXAMPLE: Bytes = Bytes::from_static(
            b"<html><input name=wresult value=result><input name=wctx value=ctx></html>",
        );

        let mut stream = pin!(stream::once(async { Ok::<_, ()>(EXAMPLE) }));

        let config = parse_wsignin(&mut stream).await.unwrap();

        assert_eq!(config.result, "result");
        assert_eq!(config.ctx, "ctx");
    }

    #[tokio::test]
    async fn test_more() {
        const EXAMPLE: Bytes = Bytes::from_static(
            b"<html><input name=wresult value=a><input name=wctx value=b><input name=wresult value=c><input name=wctx value=d></html>",
        );

        let mut stream = pin!(stream::once(async { Ok::<_, ()>(EXAMPLE) }));

        let config = parse_wsignin(&mut stream).await.unwrap();

        assert_eq!(config.result, "c");
        assert_eq!(config.ctx, "d");
    }

    #[tokio::test]
    async fn test_empty_result() {
        const EXAMPLE: Bytes = Bytes::from_static(b"<html><input name=wctx value=ctx></html>");

        let mut stream = pin!(stream::once(async { Ok::<_, ()>(EXAMPLE) }));

        let config = parse_wsignin(&mut stream)
            .await
            .expect_err("No wresult in HTML");

        assert!(matches!(config, WSignInParseError::MissingData));
    }

    #[tokio::test]
    async fn test_empty_ctx() {
        const EXAMPLE: Bytes =
            Bytes::from_static(b"<html><input name=wresult value=result></html>");

        let mut stream = pin!(stream::once(async { Ok::<_, ()>(EXAMPLE) }));

        let config = parse_wsignin(&mut stream)
            .await
            .expect_err("No wctx in HTML");

        assert!(matches!(config, WSignInParseError::MissingData));
    }

    #[tokio::test]
    async fn test_empty_both() {
        const EXAMPLE: Bytes = Bytes::from_static(b"<html></html>");

        let mut stream = pin!(stream::once(async { Ok::<_, ()>(EXAMPLE) }));

        let config = parse_wsignin(&mut stream)
            .await
            .expect_err("No wsignin in HTML");

        assert!(matches!(config, WSignInParseError::MissingData));
    }
}
