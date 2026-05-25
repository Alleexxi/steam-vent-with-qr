use std::future::Future;
use tokio::io::{AsyncWrite, AsyncWriteExt, stdout, Stdout};

/// A trait for displaying the QR challenge URL during a QR-code login flow.
///
/// The handler is called with the initial challenge URL as soon as the QR
/// session begins, and again whenever Steam rotates the URL (the challenge
/// is short-lived and must be re-rendered periodically until the user
/// approves the login in the Steam mobile app).
///
/// The library comes with a built-in [`ConsoleQrChallengeHandler`] that
/// prints the URL to stdout. Apps that want to render an actual QR code,
/// display it in a GUI, or push it through another channel can implement
/// this trait themselves.
pub trait QrChallengeHandler: Send {
    /// Called with the initial challenge URL and again whenever Steam
    /// rotates it.
    fn handle_challenge_url(
        &mut self,
        challenge_url: &str,
    ) -> impl Future<Output = ()> + Send;

    /// Called once when Steam first reports that the user has interacted
    /// with the QR (scanned but not yet approved). Default impl is a no-op.
    fn handle_remote_interaction(&mut self) -> impl Future<Output = ()> + Send {
        async {}
    }
}

/// Print the QR challenge URL to stdout.
///
/// Renders no actual QR code — applications that want a visible QR code
/// should implement [`QrChallengeHandler`] themselves and use a crate such
/// as `qrcode` to render the URL.
pub type ConsoleQrChallengeHandler = WriterQrChallengeHandler<Stdout>;

/// Print the QR challenge URL to an arbitrary async writer.
pub struct WriterQrChallengeHandler<Write> {
    output: Write,
}

impl Default for ConsoleQrChallengeHandler {
    fn default() -> Self {
        WriterQrChallengeHandler { output: stdout() }
    }
}

impl<Write> WriterQrChallengeHandler<Write>
where
    Write: AsyncWrite + Unpin + Send + Sync,
{
    pub fn new(output: Write) -> Self {
        WriterQrChallengeHandler { output }
    }
}

impl<Write> QrChallengeHandler for WriterQrChallengeHandler<Write>
where
    Write: AsyncWrite + Unpin + Send + Sync,
{
    async fn handle_challenge_url(&mut self, challenge_url: &str) {
        let msg = format!(
            "scan this URL with the Steam mobile app:\n  {challenge_url}\n"
        );
        self.output.write_all(msg.as_bytes()).await.ok();
        self.output.flush().await.ok();
    }

    async fn handle_remote_interaction(&mut self) {
        self.output
            .write_all(b"scanned, waiting for approval...\n")
            .await
            .ok();
        self.output.flush().await.ok();
    }
}
