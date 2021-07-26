// git fetch --no-tags --all -p &&
// (git merge --ff-only || true) &&
// git branch -vv | grep ': gone]' | awk '{print $1}' | xargs -n 1 git branch -D &&
// if [ "`git rev-parse --abbrev-ref HEAD`" != "staging" ]; then
//     git fetch origin staging:staging
// fi

use git2::{AutotagOption, Cred, FetchOptions, RemoteCallbacks, Repository};
use std::io::{self, Write};
use std::{env, str};
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

    let remote = repo
        .find_remote(remote)
        .or_else(|_| repo.remote_anonymous(remote))?;

    fetch_remote(remote)?;
    merge(repo)?;

    Ok(())
}

fn fetch_remote(mut remote: git2::Remote) -> Result<(), git2::Error> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.sideband_progress(git_sideband_progress_cb);
    callbacks.credentials(git_credentials_cb);
    callbacks.update_tips(git_update_tips_cb);
    callbacks.transfer_progress(git_transfer_progress_cb);

    let mut options = FetchOptions::new();
    options.remote_callbacks(callbacks);
    options.download_tags(git2::AutotagOption::None);
    options.prune(git2::FetchPrune::On);
    remote.download(&[] as &[&str], Some(&mut options))?;

    {
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

    let list = remote.list()?;
    for head in list {
        let local_or_remote = match head.is_local() {
            false => "remote",
            true => "local",
        };
        println!("remote name: {} is {}", head.name(), local_or_remote);
    }

    remote.update_tips(None, true, AutotagOption::Unspecified, None)?;
    remote.disconnect()
}

fn git_sideband_progress_cb(data: &[u8]) -> bool {
    print!("remote: {}", str::from_utf8(data).unwrap());
    io::stdout().flush().unwrap();
    true
}

fn git_credentials_cb(
    _user: &str,
    username_from_url: Option<&str>,
    _allowed_types: git2::CredentialType,
) -> Result<git2::Cred, git2::Error> {
    Cred::ssh_key(
        username_from_url.unwrap(),
        None,
        std::path::Path::new(&format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
        None,
    )
}

fn git_update_tips_cb(refname: &str, a: git2::Oid, b: git2::Oid) -> bool {
    if a.is_zero() {
        println!("[new]     {:20} {}", b, refname);
    } else {
        println!("[updated] {:10}..{:10} {}", a, b, refname);
    }
    true
}

fn git_transfer_progress_cb(stats: git2::Progress) -> bool {
    if stats.received_objects() == stats.total_objects() {
        print!(
            "Resolving deltas {}/{}\r",
            stats.indexed_deltas(),
            stats.total_deltas()
        );
    } else if stats.total_objects() > 0 {
        print!(
            "Received {}/{} objects ({}) in {} bytes\r",
            stats.received_objects(),
            stats.total_objects(),
            stats.indexed_objects(),
            stats.received_bytes()
        );
    }
    io::stdout().flush().unwrap();
    true
}

fn merge(repo: Repository) -> Result<(), git2::Error> {
    let mut options = git2::MergeOptions::new();
    options.fail_on_conflict(true);
    repo.merge(&[], Some(&mut options), None)
}

fn main() {
    let args = Args::from_args();
    match run(&args) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e),
    }
}
