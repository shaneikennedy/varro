---
title: Rebase, don't merge!
date: "2021-10-28"
description: "Let's talk about the benefits of using git rebase for integrating the latest changes from master/main into your feature branches"
---

Co-authored by [Melissa Kennedy](https://github.com/Melissa-Kennedy)

Use whatever you and your team like, I don't really care :)

But if you've never heard of rebase, or just have never used it and want to know about it, then let's start with a quick intro!

*Rebase*: _Re_-applying commits on some _base_ -> Re-base

Git's rebase is a powerful command with a few different use cases, but in this post we're just going to be talking about using rebase to update your branch with the latest changes from master/main (or any other branch).

### When to update your branch
In general it's good to keep your feature branch as close to the master or main branch as possible to make sure that when your PR is approved and your changes land in the master/main branch there are no surprises, but also to catch merge conflicts early. Most likely, you _need_ to update your branch with the latest changes from master/main because you got a little warning on your PR that you have merge conflicts and they need to be solved before you can merge your changes.

### Updating your branch with `merge`
If you're not using rebase already it probably means you're using `git merge main` (or `git pull origin main`) to update your feature branch. Merging master/main into your feature branch will create a "merge commit" that contains the latest changes, and from there on you can continue writting code for your feature.

#### Example
Let's say we have a main branch, a "feature1" branch, and a"feature2" branch. We're going to merge feature1 into master first (pretend someone on your team worked on feature1 and just merged their PR into main). Now, as the authors of feature2, we want to update our branch with the latest changes in the main branch:

###### Main branch after merging feature1

``` shell
*   be23b51 - (HEAD -> main) Merge branch 'feature1' (3 seconds ago) <github admin>
|\
| * 8e6415b - (feature1) commit from feature1 (26 minutes ago) <some other person>
|/
* 2bd5441 - (origin/main, origin/HEAD) Initial commit (4 hours ago) <some person>
```

###### feature2 branch before merging in the main branch

``` shell
* b21ccfa - (HEAD -> feature2) commit from feature2 (30 minutes ago) <some person>
* 2bd5441 - (origin/main, origin/HEAD) Initial commit (4 hours ago) <some person>
```

###### feature2 branch after merging in the main branch (`git merge main`)

``` shell
*   13c9d7e - (HEAD -> feature2) Merge branch 'main' into feature2 (3 seconds ago) <some person>
|\
| *   be23b51 - (main) Merge branch 'feature1' (2 minutes ago) <github admin>
| |\
| | * 8e6415b - (feature1) commit from feature1 (28 minutes ago) <some other person>
| |/
* / b21ccfa - commit from feature2 (26 minutes ago) <some person>
|/
* 2bd5441 - (origin/main, origin/HEAD) Initial commit (4 hours ago) <some person>
```

> The outputs above are from `git log --graph`. I find this makes it easier to visualize what's actually going on in the git tree compared to the default `git log`

Now we can continue commiting on feature2 with the latest changes from main.

### Updating your branch using `rebase`
To be clear, updating your feature branches using merge or rebase **accomplish the same thing**, but it works in a different way. Remember "rebase" means re-applying commits from one branch on top of some _base_ branch. What we're going to see here using the same example from above is re-applying the commits from feature2 on top of the main branch (after merging feature1 into main).

###### Main branch after merging feature1
``` shell
*   be23b51 - (HEAD -> main) Merge branch 'feature1' (3 seconds ago) <github admin>
|\
| * 8e6415b - (feature1) commit from feature1 (26 minutes ago) <some other person>
|/
* 2bd5441 - (origin/main, origin/HEAD) Initial commit (4 hours ago) <some person>
```

###### feature2 branch before rebasing on top of the main branch

``` shell
* b21ccfa - (HEAD -> feature2) commit from feature2 (30 minutes ago) <some person>
* 2bd5441 - (origin/main, origin/HEAD) Initial commit (4 hours ago) <some person>
```

###### feature2 branch after rebasing on to the main branch (git rebase main)

``` shell
* a8fa9fb - (HEAD -> feature2) commit from feature2 (2 seconds ago) <some person>
*   be23b51 - (main) Merge branch 'feature1' (12 minutes ago) <github admin>
|\
| * 8e6415b - (feature1) commit from feature1 (38 minutes ago) <some other person>
|/
* 2bd5441 - (origin/main, origin/HEAD) Initial commit (4 hours ago) <some person>
```

> Note that the **latest** commit in feature2 _after_ rebasing is the commit that was unique to feature2. This holds if we had multiple commits in feature2, they would all be placed *after* the "Merge branch 'feature1'..." commit

It is **very important** that we notice that the commit hash from "commit from feature2" has changed after rebasing. Before the rebase, that commit had the hash b21ccfa and after the hash is a8fa9fb. This happens because a commit hash exists as a reference to changes, but remember, commits are changes _relative_ to the previous history. When we rebase, we are replacing the history of branch "feature2" with what is currently in the "main" branch. Because this history has changed and the fact that a commit hash is a reference to changes relative to a previous history, the commit hash for "commit from feature2" **_must_** be different after rebasing.

### So why should I use rebase instead of merge?
1. Cleaner git log that is easier to reason about
2. Commits in PRs are only changes that you made
3. Merge conflicts aren't hidden in a merge commit, they never existed in the first place

Let's go through these:

### 1. Cleaner git log that is easier to reason about

###### Updating feature2 with rebase
``` shell
* a8fa9fb - (HEAD -> feature2) commit from feature2 (2 seconds ago) <some person>
*   be23b51 - (main) Merge branch 'feature1' (12 minutes ago) <github admin>
|\
| * 8e6415b - (feature1) commit from feature1 (38 minutes ago) <some other person>
|/
* 2bd5441 - (origin/main, origin/HEAD) Initial commit (4 hours ago) <some person>
```

###### Updating feature2 with merge

``` shell
*   13c9d7e - (HEAD -> feature2) Merge branch 'main' into feature2 (3 seconds ago) <some person>
|\
| *   be23b51 - (main) Merge branch 'feature1' (2 minutes ago) <github admin>
| |\
| | * 8e6415b - (feature1) commit from feature1 (28 minutes ago) <some other person>
| |/
* / b21ccfa - commit from feature2 (26 minutes ago) <some person>
|/
* 2bd5441 - (origin/main, origin/HEAD) Initial commit (4 hours ago) <some person>
```

When you look at the changes being made in a feature branch, we typically look at them relative to the main branch: as changes that we want to _add_ to the main branch. When I look at feature2 after updating with merge, I see a bunch of commit and branch information related to changes in main and the feature1 branch, and then at the bottom I see my changes. This is not intuitive to me. Feature2 should only concern my changes that I want to get merged in to main, not someone else's feature1 work. This problem is exacerbated when the feature1 branch has many more commits in it or if feature1 wasn't the only branch merged into main since feature2 branched away from main. **The more out of date feature2 is, the worse this problem becomes.**

**So why can't I just merge master into my branch often?**

### 2. Commits in PRs are only changes that you made
When I'm reviewing your PR, I'm looking at the commits. I want to see what changes you made to the code. I do not need to see that you updated your branch with the lastest changes in main; this information is not relavent to the changes that you're trying to get in to master/main. Since PRs are relative to some target branch (usually master/main) and rebasing places your commits on top of master/main, you only see the commits for changes you made.

This is from a currently open PR in apache airflow:

![Merge commits](/images/merge-commits.png "merge commits")

> My complete and utter respect to anyone helping build OSS, especially airflow which I've used and loved. This is not meant to demean this PR author, rather to show an example of what I prefer not to see as a reviewer. After all, rebase vs merge is 100% a preference debate.

### 3. Merge conflicts aren't hidden in a merge commit, they never existed in the first place

Last and certainly not least, I really prefer the way rebasing handles merge conflicts compared to how merge handles them. When you update your feature branch with the latest changes from main and you happen to have merge conflicts, git will ask you to resolve them and then run `git commit`. What this is doing is making that merge commit into a commit that now has both the changes from main that you're trying to pull in *_and_* the conflict resolutions. The commit message will have the typical "Merge ..." and in this case it will include some information in the details that conflicts were resolved.

First off, as a PR reviewer or someone looking through the git history, it is not relavent that there were merge conflicts at one point in this feature branch. And secondly, as a PR reviewer the fact that the conflict resolution changes are somewhat hidden in the merge commit means I can't just assume that any merge commit in your feature branch is just pulling in the latest changes; I now need to assume that any time there is a merge commit, there might be some conflict resolution changes that I need to take a look at.

Rebasing handles this process much differently, and much better in my opinion. When rebasing your feature branch on to main, it will do so by applying one commit at a time and if merge conflicts are encountered the rebase will pause, allow you to resolve the conflicts as if they were a part of the changes you made in the first place, and then allow you to continue your rebase (`git rebase --continue`), repeating this process until all of your commits have been applied.

What this does instead of registering a commit indicating that there were merge conflicts at one point, like git merge does, rebase makes it as if there never were any merge conflicts; as if you've always been working off of the latest changes from main. This keeps your PRs tidy with just the changes you're trying to merge into master, and this is what makes for a much better PR review experience, in my opinion.
