use git2::{AutotagOption, FetchOptions, FetchPrune, Remote, RemoteCallbacks, Repository};
use crate::repo::git_credentials_callback;

pub fn do_fetch<'a>(repo: &'a Repository, refs: &[&str], remote: &'a mut Remote) -> Result<git2::AnnotatedCommit<'a>, git2::Error> {

    let mut cb = RemoteCallbacks::new();

    cb.credentials(git_credentials_callback);

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);

    fo.download_tags(AutotagOption::All);
    fo.update_fetchhead(true);

    remote.fetch(refs, Some(&mut fo), None)?;

    let fetch_head = repo.find_reference(&format!("refs/remotes/origin/{}", refs[0]))?;
    Ok(repo.reference_to_annotated_commit(&fetch_head).unwrap())
}

fn fast_forward(repo: &Repository, lb: &mut git2::Reference, rc: &git2::AnnotatedCommit, ) -> Result<(), git2::Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(
        git2::build::CheckoutBuilder::default()
            // For some reason the force is required to make the working directory actually get updated
            // I suspect we should be adding some logic to handle dirty working directory states
            // but this is just an example so maybe not.
            .force(),
    ))?;
    Ok(())
}

fn normal_merge(repo: &Repository, local: &git2::AnnotatedCommit, remote: &git2::AnnotatedCommit, ) -> Result<(), git2::Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(());
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    // now create the merge commit
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    // Do our merge commit and set current branch head to that commit.
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    // Set working tree to match head.
    repo.checkout_head(None)?;
    Ok(())
}

pub fn do_merge<'a>(repo: &'a Repository, remote_branch: &str, fetch_commit: git2::AnnotatedCommit<'a>) -> Result<String, git2::Error> {
    let mut msg: &str = "";
    // 1. do a merge analysis
    let analysis = repo.merge_analysis(&[&fetch_commit])?;

    // 2. Do the appopriate merge
    if analysis.0.is_fast_forward() {
        msg = "Doing a fast forward";
        // do a fast forward
        let refname = format!("refs/heads/{}", remote_branch);
        match repo.find_reference(&refname) {
            Ok(mut r) => {
                fast_forward(repo, &mut r, &fetch_commit)?;
            }
            Err(_) => {
                // The branch doesn't exist so just set the reference to the
                // commit directly. Usually this is because you are pulling
                // into an empty repository.
                repo.reference(
                    &refname,
                    fetch_commit.id(),
                    true,
                    &format!("Setting {} to {}", remote_branch, fetch_commit.id()),
                )?;
                repo.set_head(&refname)?;
                repo.checkout_head(Some(
                    git2::build::CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
            }
        };
    } else if analysis.0.is_normal() {
        // do a normal merge
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(repo, &head_commit, &fetch_commit)?;
    } else {
        msg = "Nothing to do...";
    }
    Ok(msg.to_string())
}

pub fn fetch_repository_from_remote(remote_name: &str, remote_branch: &str, repository: &Repository) -> Result<String, git2::Error> {
    let mut remote = repository.find_remote(remote_name).unwrap();

    let result = if let Ok(fetch_commit) =  do_fetch(repository, &[remote_branch], &mut remote) {
        do_merge(repository, remote_branch, fetch_commit)
    } else {
        Err(git2::Error::from_str("Could not find remote branch!"))
    };
    result
}

pub fn fetch_branches_repository_from_remote(remote_name: &str, repository: &Repository) -> Result<String, git2::Error> {
    match repository.find_remote(remote_name) {
        Ok(mut remote) => fetch_all(&mut remote),
        Err(e) => Err(git2::Error::from_str(e.message()))
    }
}

pub fn fetch_all(remote: &mut Remote) -> Result<String, git2::Error> {
    let mut cb = RemoteCallbacks::new();
    cb.credentials(git_credentials_callback);
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);
    fo.prune(FetchPrune::On);
    remote.fetch(&[] as &[&str], Some(&mut fo), Some("updating local"))?;
    remote.update_tips(None, true, AutotagOption::Unspecified, None)?;
    Ok(String::from("Fetching is done!"))
}