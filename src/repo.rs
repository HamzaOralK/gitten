use git2::{Cred, CredentialType, Repository};
use std::path::{Path, PathBuf};

pub fn git_credentials_callback(
    _url: &str,
    user_from_url: Option<&str>,
    cred_types_allowed: CredentialType,
) -> Result<Cred, git2::Error> {
    let user = if let Some(usr) = user_from_url {
        usr
    } else {
        return Err(git2::Error::from_str("no credential option available"))
    };

    if cred_types_allowed.contains(CredentialType::SSH_KEY) {
        let private_key = dirs::home_dir().unwrap().join(".ssh").join("id_rsa");
        let cred = Cred::ssh_key(user, None, &private_key, None);
        return cred;
    }

    Err(git2::Error::from_str("no credential option available"))
}

pub fn is_repository(path: PathBuf) -> bool {
    match Repository::open(path) {
        Ok(_repo) => true,
        _error => false,
    }
}

pub fn get_repository(path: &PathBuf) -> Option<Repository> {
    match Repository::open(path) {
        Ok(repo) => Some(repo),
        Err(_e) => None,
    }
}

pub fn get_repository_tags(repository: &Option<Repository>) -> Vec<String> {
    let mut tags = Vec::new();
    if let Some(r) = repository {
        r.tag_names(None).iter().for_each(|f| {
            f.iter().for_each(|x| {
                if let Some(tag) = x {
                    tags.push(tag.to_string());
                };
            });
        });
    }
    tags
}

pub fn get_repository_branches(repository: &Option<Repository>) -> Vec<String> {
    let mut branches_string = Vec::new();

    if let Some(r) = repository {
        let branches = match r.branches(None) {
            Ok(branches) => Some(branches),
            Err(_) => None,
        };

        branches.unwrap().for_each(|b| {
            let b1 = b.unwrap().0.name().unwrap().unwrap().to_string();
            branches_string.push(b1);
        });
    }
    branches_string
}

pub fn get_repository_active_branch(repository: &Option<Repository>) -> String {
    let mut branch_id: String = "".to_string();
    if let Some(r) = repository {
        if let Ok(head) = r.head() {
            if let Some(name) = head.name() {
                branch_id = name.replace("refs/heads/", "")
            }
        } else {
            branch_id = "empty".to_string()
        }
    }
    branch_id
}

pub fn get_files_changed(repository: &Option<Repository>) -> Option<usize> {
    if let Some(r) = repository {
        return match r.diff_index_to_workdir(None, None) {
            Ok(diff) => Some(diff.stats().unwrap().files_changed()),
            Err(_e) => None,
        };
    } else {
        None
    }
}
