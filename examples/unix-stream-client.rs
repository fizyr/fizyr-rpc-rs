use fizyr_rpc::UnixStreamPeer;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
#[structopt(setting = structopt::clap::AppSettings::UnifiedHelpMessage)]
#[structopt(setting = structopt::clap::AppSettings::DeriveDisplayOrder)]
struct Options {
	socket: PathBuf,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
	if let Err(e) = do_main(&Options::from_args()).await {
		eprintln!("Error: {}", e);
		std::process::exit(1);
	}
}

async fn do_main(options: &Options) -> Result<(), String> {
	// Connect to a remote server.
	let mut peer = UnixStreamPeer::connect(&options.socket, Default::default()).await
		.map_err(|e| format!("failed to connect to {}: {}", options.socket.display(), e))?;

	// Send a request to the remote peer.
	let mut request = peer
		.send_request(1, &b"Hello World!"[..])
		.await
		.map_err(|e| format!("failed to send request: {}", e))?;

	loop {
		let update = request
			.recv_update()
			.await
			.map_err(|e| format!("failed to read message: {}", e))?;
		let update = match update {
			Some(x) => x,
			None => break,
		};
		let message = std::str::from_utf8(&update.body.data).map_err(|_| "invalid UTF-8 in update")?;
		eprintln!("Received update: {}", message);
	}

	let response = request
		.recv_response()
		.await
		.map_err(|e| format!("failed to read message: {}", e))?;
	// Parse the message body as UTF-8, print it and exit the loop.
	let message = std::str::from_utf8(&response.body.data).map_err(|_| "invalid UTF-8 in response")?;
	eprintln!("Received response: {}", message);

	Ok(())
}
