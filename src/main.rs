// git fetch --no-tags --all -p &&
// (git merge --ff-only || true) &&
// git branch -vv | grep ': gone]' | awk '{print $1}' | xargs -n 1 git branch -D &&
// if [ "`git rev-parse --abbrev-ref HEAD`" != "staging" ]; then
//     git fetch origin staging:staging
// fi

use git2::{AutotagOption, Cred, FetchOptions, RemoteCallbacks, Repository};
use log::*;
use regex::Regex;
use std::{env, str};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(name = "remote")]
    arg_remote: Option<String>,
}

fn run(args: &Args) -> Result<(), git2::Error> {
    let mut repo = Repository::open(".")?;
    let remote = args.arg_remote.as_ref().map(|s| &s[..]).unwrap_or("origin");

    info!("Fetching {} for repo", remote);

    fetch_remote(&mut repo, remote)?;
    merge(&mut repo)?;
    // prune_deleted_branches(&mut repo)?;
    fetch_origin_head(&mut repo, remote)?;

    Ok(())
}

fn fetch_remote(repo: &mut git2::Repository, remote: &str) -> Result<(), git2::Error> {
    let mut remote = repo
        .find_remote(remote)
        .or_else(|_| repo.remote_anonymous(remote))?;

    let mut callbacks = RemoteCallbacks::new();
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
            info!(
                "Received {}/{} objects in {} bytes (used {} local objects)\n",
                stats.indexed_objects(),
                stats.total_objects(),
                stats.received_bytes(),
                stats.local_objects()
            );
        } else {
            info!(
                "Received {}/{} objects in {} bytes\n",
                stats.indexed_objects(),
                stats.total_objects(),
                stats.received_bytes()
            );
        }
    }

    remote.update_tips(None, true, AutotagOption::Unspecified, None)?;
    remote.disconnect()
}

fn fetch_origin_head(repo: &mut git2::Repository, remote: &str) -> Result<(), git2::Error> {
    let mut remote = repo
        .find_remote(remote)
        .or_else(|_| repo.remote_anonymous(remote))?;

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(git_credentials_cb);
    remote.connect_auth(git2::Direction::Fetch, Some(callbacks), None)?;

    let default = remote.default_branch()?;
    let branch = default.as_str().unwrap();
    let re = Regex::new(r"^(refs/heads/)(.*)$").unwrap();
    let captures = re.captures(branch).unwrap();

    remote.fetch(&[&captures[2]], None, None)?;
    remote.disconnect()
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
        info!("[new]     {:20} {}\n", b, refname);
    } else {
        info!("[updated] {:10}..{:10} {}\n", a, b, refname);
    }
    true
}

fn git_transfer_progress_cb(stats: git2::Progress) -> bool {
    if stats.received_objects() == stats.total_objects() {
        info!(
            "Resolving deltas {}/{}\n",
            stats.indexed_deltas(),
            stats.total_deltas()
        );
    } else if stats.total_objects() > 0 {
        info!(
            "Received {}/{} objects ({}) in {} bytes\n",
            stats.received_objects(),
            stats.total_objects(),
            stats.indexed_objects(),
            stats.received_bytes()
        );
    }
    true
}

fn merge(repo: &mut Repository) -> Result<(), git2::Error> {
    let mut options = git2::MergeOptions::new();
    options.fail_on_conflict(true);

    let result = repo.merge(&[], Some(&mut options), None);

    match result {
        Ok(_result) => info!("test"),
        Err(e) => error!("{:?}", e),
    };

    Ok(())
}

fn main() {
    env_logger::init();
    let args = Args::from_args();
    match run(&args) {
        Ok(()) => {}
        Err(e) => error!("error: {}", e),
    }
}
