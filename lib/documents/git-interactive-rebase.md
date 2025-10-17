---
title: Interactive rebasing, git god-mode
date: "2021-11-06"
description: "Learn how to harness the power of git's interactive rebase, a command that let's you alter the commit history with full control"
---

Imagine you have this feature branch with 5 commits:


``` text
* 02a7e84ccee - (HEAD -> billing, origin/billing) Fail gracefully if unknown error (8 hours ago) <Commit author>
* 8647daf1a2f -  Add R0801 to pylint ignore list (8 hours ago) <Commit author>
* 1c5d459147a -  Don't use BaseException anywhere (8 hours ago) <Commit author>
* 62de8bb854d -  Send billing events (8 hours ago) <Commit author>
* d783109d594 -  Raise insuffient data exceptions (8 hours ago) <Commit author>
* d8d58d1f055 -  Update event service client with billing logic (8 hours ago) <Commit author>
```

You've put the work in to reflect incremental units of change, each with their own value, as well authored commits and you open a PR. Overall the PR looks good, but your reviewer found a few issues:

1. Typo here
2. Missing edge case there
3. Missing test somewhere else

What we could do is create a commit that fixes the typo, handles the edge case and adds a test called "Update with PR feedback", but I'm here to tell you that we can do better than that. A commit that fixes mistakes that were introduced in this PR does not make any sense. In the git history we would just prefer that these mistakes never existed at all. So let me propose a different way:

Let's add a new commit for each fix we make
* Fix typo
* Handle edge case
* Add test

These are bad commit messages but you'll see why in a second. Further, let's say we know that the typo happened in  "d8d58d1f055 -  Update event service client with billing logic", the edge case was missed in "d783109d594 -  Raise insuffient data exceptions", and that you missed a test in "62de8bb854d -  Send billing events".

So now our commit history looks like this :

``` text
* da7f3059367 - (HEAD -> billing) Add test (1 second ago) <Commit author>
* e62d6fd4bab - Handle edge case (38 seconds ago) <Commit author>
* 01507911dfd - Fix typo (2 minutes ago) <Commit author>
* 02a7e84ccee - (origin/billing) Fail gracefully if unknown error (8 hours ago) <Commit author>
* 8647daf1a2f - Add R0801 to pylint ignore list (8 hours ago) <Commit author>
* 1c5d459147a - Don't use BaseException anywhere (8 hours ago) <Commit author>
* 62de8bb854d - Send billing events (8 hours ago) <Commit author>
* d783109d594 - Raise insuffient data exceptions (8 hours ago) <Commit author>
* d8d58d1f055 - Update event service client with billing logic (8 hours ago) <Commit author>
```

What we're going to do here is combine, or fixup, these small fixes with the commit where the problem was introduced; this means I'm going to combine  "01507911dfd - Fix typo (2 minutes ago)" with "d8d58d1f055 -  Update event service client with billing logic" and so on.

#### Rebase recap

If you haven't already read my [intro to git rebase](/git-rebase), that's a good place to start in order to understand rebasing in general. TL;DR rebasing is to _re_ apply commits on top of some _base_. In my other article we talked about rebasing feature branches on top of the master/main branch, but here we just want to modify the commit history that's unique to our branch (the commit listed above), so in this case, we want our _base_ to be whatever comes before our earliest commit. Let's learn by doing and rebase this branch -- interactively:

> Command: `git rebase <options> _base_` where base is what you want to re apply commits on to. In my previous post, we wanted to rebase on top of master/main so we would write `git rebase origin/main`. But here we just want to rebase on top of what came before our earliest commit: `git rebase <commit-sha>` but this can be a bit tideous to do, so I prefer to use `git rebase HEAD~n` where n is the number of commits back to the base commit.

#### --interactive or -i

I have 9 commits in this branch, which means the base that I want is 10 commits back:

`git rebase -i HEAD~9`

``` text
pick d8d58d1f055 Update event service client with billing logic
pick d783109d594 Raise insuffient data exceptions
pick 62de8bb854d Send billing events
pick 1c5d459147a Don't use BaseException anywhere
pick 8647daf1a2f Add R0801 to pylint ignore list
pick 02a7e84ccee Fail gracefully if unknown error
pick 01507911dfd Fix typo
pick e62d6fd4bab Handle edge case
pick da7f3059367 Add test

#
# Commands:
# p, pick <commit> = use commit
# r, reword <commit> = use commit, but edit the commit message
# e, edit <commit> = use commit, but stop for amending
# s, squash <commit> = use commit, but meld into previous commit
# f, fixup [-C | -c] <commit> = like "squash" but keep only the previous
#                    commit's log message, unless -C is used, in which case
#                    keep only this commit's message; -c is same as -C but
#                    opens the editor
# x, exec <command> = run command (the rest of the line) using shell
# b, break = stop here (continue rebase later with 'git rebase --continue')
# d, drop <commit> = remove commit
# l, label <label> = label current HEAD with a name
# t, reset <label> = reset HEAD to a label
# m, merge [-C <commit> | -c <commit>] <label> [# <oneline>]
# .       create a merge commit using the original merge commit's
# .       message (or the oneline, if no original merge commit was
# .       specified); use -c <commit> to reword the commit message
#
# These lines can be re-ordered; they are executed from top to bottom.
#
# If you remove a line here THAT COMMIT WILL BE LOST.
#
# However, if you remove everything, the rebase will be aborted.
#

```

This is what you should see in your git commit editor. Let me point out some interesting information:

1. Next to all of the commits we have the word "pick"
2. The command list that includes the various options we have and their explanations
3. This is an editable file: we can change whether we "pick" certain commits, we can move commits up or down etc.

Looking at the descriptions for each command:

* Pick - "use commit" this means leave it as is
* Reword - don't like the commit message you used?
* Edit - forgot something on this commit?
* Squash - "meld" into previous commit (this is what we want to do)
* Fixup - same as squash, but use the first commit's message (this is also what we want, and what we'll use because I like the commit messages I made initially)
* ...

You get the point, I find these first four to be the most used in my workflow but the other options are worth knowing about.

Back to our rebase!

I want to take my small fixup commits, and "meld" them onto the commit where they were introduced:

``` text
pick cc449795730 Update event service client with billing logic
fixup bd8222bd083 Fix typo
pick a96ad17080a  Raise insuffient data exceptions
fixup 9c8776fbe9f Handle edge case
pick 745554d6aa5 Send billing events
fixup 3eef03eb869 Add test
pick a5487033dbb Don't use BaseException anywhere
pick aaebfa866a2 Add R0801 to pylint ignore list
pick ab307ed1550 Fail gracefully if unknown error
```

Your rebase file should now look like this after you moved the fixup commits one spot after where the thing they're fixing was introduced, and then changed "pick" to "fixup" because we don't want those little fixup commits in our git history.

Now save and close the file and run `git log --graph`

> --graph is a preference, use normal git log if you want

``` text
* 9857802cbb1 - (HEAD -> billing) Fail gracefully if unknown error (5 seconds ago) <Commit author>
* 747321db1e0 -  Add R0801 to pylint ignore list (6 seconds ago) <Commit author>
* cbd6a8442b7 -  Don't use BaseException anywhere (6 seconds ago) <Commit author>
* 6f1f0da8a9d -  Send billing events (6 seconds ago) <Commit author>
* 158da739da5 -  Raise insuffient data exceptions (6 seconds ago) <Commit author>
* 4f702d469e3 -  Update event service client with billing logic (6 seconds ago) <Commit author>
```

And there you go! Those fixup commits are gone, but their changes are in the commits that they were squashed onto (use `git show <sha>` to see for yourself).

Some important things to notice: similar to rebasing on top of master like in my last post, you'll see that all of the commit hashes have changed, even for the commits that weren't changed. Go read that previous post for the explanation why.

The time stamps were modifed as well: before most of my commits were from 8 hours ago, and now it's saying 6 seconds ago. This is because they're new commits.
