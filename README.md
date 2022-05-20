# Alfred
Alfred is git project manager with multiple repositories.

With alfred you can check out to new branches and tag branches from you active.

## Run
```shell
cargo run <path to open in alfred with git projects inside>
```

## Usage
To change window use "r" for Repositories, "t" for Tags and "B" for branches.

And right now while you are on repositories you can do:

Checkout to a new or already in place branch:
```shell
:co <branch-name>
```

Create a tag from active branch:
```shell
:tag <tag-name>
```

While you are on a branch or a tag you can do the following to push the 
selected branch or tag to the desired remote:
```shell
:p <remote-name>
```

## Showcase
![](alfred.gif)

## TODO
- [X] Push to remote with tags or not
- [ ] Mark dirty head and allow to reset
- [ ] Force pull
- [ ] Pull
- [ ] Multiple repository selection and bulk branch and tag creation
