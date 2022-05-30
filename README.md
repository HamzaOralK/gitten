# Alfred
Alfred is git project manager with multiple repositories.

With alfred you can check out to new branches and tag branches from you active.

## Prerequisite
To build one has to have OpenSSL installed and adjust config.toml file accordingly to be able to run and 
compile the project.

For mac:
```shell
brew install openssl@3
```

## Run
```shell
cargo run <path to open in alfred with git projects inside>
```

## Usage
To change window use "r" for Repositories, "t" for Tags and "B" for branches.

And right now while you are on repositories you can do:

Fetch a remote for a repository
```shell
:fetch <remote-name>
```

Checkout to a new or already in place branch:
```shell
:co <branch-name>
```

Pull from the current selected branch from the supplied remote:
```shell
:pull <remote-name>
```

Create a tag from active branch:
```shell
:tag <tag-name>
```

While you are on a branch or a tag you can do the following to push the 
selected branch or tag to the desired remote:
```shell
:push <remote-name>
```

To search within the selections:
```shell
/search <search-string>
```

To run a command with the selected repository path:
```shell
$<command-string>
```

This I use to open a path in the desired application,
```shell
$code

or 

$idea

or

$open
```


## Showcase
![](alfred.gif)

## TODO
- [X] Push to remote with tags or not
- [X] Mark dirty head and allow to reset
- [X] Fetch remote
- [X] Force pull
- [X] Search within the selections
- [X] Run commands with the path
- [ ] Add tests
