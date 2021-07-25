// git fetch --no-tags --all -p &&
// (gff || true) &&
// git branch -vv | grep ': gone]' | awk '{print $1}' | xargs -n 1 git branch -D &&
// if [ "`git rev-parse --abbrev-ref HEAD`" != "staging" ]; then
//     git fetch origin staging:staging
// fi

use git2::{Repository, RemoteCallbacks, FetchOptions};
use std::io::{self, Write};
use std::str;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(name = "remote")]
    arg_remote: Option<String>,
}

fn run(args: &Args) -> Result<(), git2::Error> {
    let repo = Repository::open(".")?;
    let remote = args.arg_remote.as_ref().map(|s| &s[..]).unwrap_or("origin");

    // Figure out whether it's a named remote or a URL
    println!("Fetching {} for repo", remote);

    let mut cb = RemoteCallbacks::new();
    let mut remote = repo
        .find_remote(remote)
        .or_else(|_| repo.remote_anonymous(remote))?;

    cb.sideband_progress(|data| {
        print!("remote: {}", str::from_utf8(data).unwrap());
        io::stdout().flush().unwrap();
        true
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);
    remote.download(&[] as &[&str], Some(&mut fo))?;

    {
        // If there are local objects (we got a thin pack), then tell the user
        // how many objects we saved from having to cross the network.
        let stats = remote.stats();
        if stats.local_objects() > 0 {
            println!(
                "\rReceived {}/{} objects in {} bytes (used {} local \
                 objects)",
                stats.indexed_objects(),
                stats.total_objects(),
                stats.received_bytes(),
                stats.local_objects()
            );
        } else {
            println!(
                "\rReceived {}/{} objects in {} bytes",
                stats.indexed_objects(),
                stats.total_objects(),
                stats.received_bytes()
            );
        }
    }

    remote.disconnect()?;

    Ok(())
}

fn main() {
    let args = Args::from_args();
    match run(&args) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e),
    }
}