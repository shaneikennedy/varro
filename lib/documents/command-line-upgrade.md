---
title: Upgrade your command line
date: "2021-04-13"
description: "Some tips, tricks and tools to help you speed up your command line workflow so you can get back focusing on your code"
---

#### Context switching in your workflow (the enemy)
Imagine you're writing a function that depends on a module in another one of your company's repositories; you probably don't have jump-to-definition capabilities here in your editor, which means you need to go find this module manually on your machine (if this sounds awful, it is. Consider this before going all in on multi-repo microservices). If every time you need to do this you need to re-remember/go look up commands, or just need to type a lot of tedious ones, it can take you right out of the context that you're currently focused on: writing code. Being slow at this isn't just annoying for you, it can be mentally draining.

#### The argument against speed - and why it makes no sense
People love to talk about how "speed doesn't matter" because "my editor/workflow isn't and shouldn't be the bottleneck when coding", and they're right. But no one is arguing that speed matters more than thinking about and designing your solution? Speed won't make your code better, but it does help with reducing lengthy context switching, and the general satisfaction of how you work, and this has been very important in my experience. There's nothing wrong with being new at something, but no one likes feeling like they're new.

Think about anything you've ever been good at: when you started you were probably slow, maybe it felt awkward, and maybe more importantly you might have been around other people who were doing the same thing as you but much, _much_ faster. Not a great feeling. But as time went by and you got better, you got comfortable, you learned some tricks, and soon enough you were much faster than before. This is just a good feeling to have, regardless of what you're doing.

#### So let's upgrade that workflow
So now you've just started as a junior dev, working 8-10 hours a day and you're feeling slow, awkward and uncomfortable at all the things _around_ coding; here are some tips, tricks and tools to help you get up to speed so you can focus on your code.

## Your shell
Not everyone likes to work in the terminal, but if you do it's nice to have a shell that helps you out.

#### [Zsh](https://en.wikipedia.org/wiki/Z_shell)
[zsh](https://en.wikipedia.org/wiki/Z_shell) is an extension of bash with some nice improvements, and is now the default shell on new apple computers. Taking it one step further is [oh-my-zsh](https://ohmyz.sh/), a framework for extending zsh with plugins and is what I personally use. This gives me a clean and helpful foundation for my terminal. Some of my favorite plugins are:
* [refined theme](https://github.com/ohmyzsh/ohmyzsh/blob/master/themes/refined.zsh-theme)
* [zsh-syntax-highlighting](https://github.com/zsh-users/zsh-syntax-highlighting)
* [zsh-autosuggestions](https://github.com/zsh-users/zsh-autosuggestions)

#### [Fish](https://fishshell.com/)
The friendly interactive shell or [fish](https://fishshell.com/) is a really interesting shell that takes a modern approach to shell configuration. Similar to oh-my-zsh there is [oh-my-fish](https://github.com/oh-my-fish/oh-my-fish) for extending this shell with plugins. The only reason I haven't switched from zsh to fish is crippling laziness.

## Command line tools
#### [z](https://github.com/rupa/z)
If you work in lots of repositories you want [z](https://github.com/rupa/z) in your toolbelt. This cli tool keeps a history of directories you visit and makes jumping between folders much easier.

#### [rg](https://github.com/BurntSushi/ripgrep)/[ag](https://github.com/ggreer/the_silver_searcher)
These are both improvements on `grep`. If you need to search for something in a project like where a certain class is defined, usages of a certain word (when your designer tells you we're no longer calling it by "x", it should now be "y"), these two are life savers. [rg](https://github.com/BurntSushi/ripgrep) and [ag](https://github.com/ggreer/the_silver_searcher) can also act as backends for some editor plugins and some other tools in this post.

#### [bat](https://github.com/sharkdp/bat)
[bat](https://github.com/sharkdp/bat) is a pretty version of cat. Just nice to have some syntax highlighting when you're viewing a files contents in the terminal.

#### [lsd](https://github.com/Peltoche/lsd)
[lsd](https://github.com/Peltoche/lsd) more colors, this is just a nicer `ls` command. I have lsd aliased to ls because I have no reason to use ls with lsd installed and I can't be bothered to change my ls muscle memory.

#### [fzf](https://github.com/junegunn/fzf)
[fzf](https://github.com/junegunn/fzf) is a command line file fuzzy-finder. Super useful if you need to find the location of a file on the command line. You can also fuzzy-find-then-open-in-editor, which is :chefs-kiss:

#### [pgcli](https://www.pgcli.com/)/[mycli](https://www.mycli.net/)
[pgcli](https://www.pgcli.com/) basically taught me sql, along with a very nice co-worker (aside, no book, cli, editor or blog post will help you improve faster than a good mentor). This provides IDE-like syntax completion for sql and is database-aware, i.e based on what database you're connected to it can provide completions for tables when joining. [pgcli](https://www.pgcli.com/) and [mycli](https://www.mycli.net/) are the same programs for postgres and mysql, respectively.


## Your editor
I'm not going to list any editors or tell you which [one](https://www.gnu.org/software/emacs/) is my favourite. All I'm going to say is that this is where the majority of your work is going to be done, so no matter what you pick, _really learn_ it. Know how to find files/symbols quickly, get used to the git client (if you like those), learn the refactoring tools where applicable, multiple cursors for quick renaming in a single file, learn how to use *macros*. I promise you, time invested in learning your editor pays dividends in time saved in the long run.

## General advice
#### Automate when it makes sense
If you find yourself writing the same command(s) over, and over again, then it's probably worth your time to automate/streamline this.

###### Aliases
There is a general concept in shells that you can alias commands ex: `alias ls="lsd"` this means whenever I type ls in my terminal, lsd will actually be executed. But lots of CLIs let you provide aliases specific to that cli. Git is one of those programs and I have the following aliases in my _.gitconfig_:

``` shell
[alias]
        co = checkout
        st = status
        lg = log --graph --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset' --abbrev-commit --date=relative
        lg2 = log --graph --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cd) %C(bold blue)<%an>%Creset' --abbrev-commit --date=short
        search = log -p --all -S
```

This allows me to write `git st` instead of `git status`, which I previously mentioned I write all the time. The two log aliases are because the default log is not intuitive to read and the basic "git log --graph" is too verbose. Lastly, git search is more of a "I can never remember this so I will keep it here". Basically if you know you committed something in the past and it's since been deleted, for example a class that got refactored, you can do `git search myclass` and it will find commits where this class was added, modified or deleted.

###### Functions
If you find yourself writing the same multiple commands in succession all the time, you might want to write a bazh, zsh, fish function. I personally don't have any in use at the moment, but it's nice to know about these when you start noticing the pattern of repeating commands in succession.

#### "Dotfile" repositories
> Dotfiles refer to files you keep in your home folder that command line tools typically look for for configuration such as ".gitconfig" ".zshrc" and are aptly-named "dotfiles".

In your career and life you're going to have multiple computers through different jobs, ones you buy yourself, and ec2 instances you rent; you don't want to rewrite your config every time. Consider version controlling your configuration files (or dotfiles) and pushing to your favourite git remote so you can access them whenever and wherever.
