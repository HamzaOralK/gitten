# Alfred
Alfred is git project manager with multiple repositories.

With alfred you can check out to new branches and tag branches from you active.

## Run
```shell
cargo run <path to open in alfred with git projects inside>
```

## Usage
To change window use "r" for Repositories, "t" for Tags and "B" for branches.

And right now while you are on only on repositories you can do:

Checkout to a new or already in place branch:
```shell
:co <branch-name>
```

Create a tag from active branch:
```shell
:tag <tag-name>
```

## Showcase
![](alfred.gif)

## TODO
- [] Multiple repository selection and bulk branch and tag creation
- [] Push to remote with tags or not
