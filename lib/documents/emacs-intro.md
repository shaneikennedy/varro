---
title: Your first taste of emacs
date: "2021-10-10"
description: "See why emacs users are diehards when it comes to their editor with a quick intro, guided configuration and quick start on editing in your first project."
---

## 2025 edit
Emacs and the world has changed quite a bit since I wrote this "intro to emacs" article. If you are still new to emacs this post is still a good start for the basics and the concepts you need to get used to in emacs, but for actually configuring emacs I'd rather send you to [configuring emacs from scratch in 2025](/blog/configuring-emacs-from-scratch-in-2025) that shouws you how to get started with all the nice new things in emacs and the community + LLM integrations. If you're a seasoned emacser and just looking for config tips head straight to [configuring emacs from scratch in 2025](/blog/configuring-emacs-from-scratch-in-2025).

## Foreword
There are a few reasons why you might be here reading this article
1. You heard emacs is an editor for "Hardcore Devs" on some thread about what editor is best
2. You know or work with an emacs user who's pretty good and think it's worth trying out
3. You pair programmed with an emacs user who was lightning quick and their answer to "how did you ..." when you inevitably asked was: "emacs"

If you're here because of 1. then let me be the first to say that this article probably isn't for you. A developers choice of editor and their skill have no correlation. I know great devs who use emacs, vim, vscode and IntelliJ/jetbrains. The ability to be a good software engineer has nothing to do with your editor.

In my opinion Emacs is for people who are obsessed with, or atleast firmly interested in, efficiency in their coding. Emacs is a good choice for these types of people because emacs is nearly unuseable out of the box and this forces it's users to customize it _completely_. This forceful way of making users create the experience they want is why, in my opinion, you see developers like the one mentioned above in 3.; you are forced to _really learn_ your editor, so when you find some sort of inefficiency while programming, you know how to set a keybinding, write a custom function, create a new tool, of look for an existing tool. And this tutorial is for people who identify like this.

So with that out of the way, let's get started.

## What is emacs?

If I had to simplify this as much as possible, I would describe emacs as a lisp interpreter with some added functionality for editing text. What do I mean by emacs being a lisp interpreter? I can open a file, write some (emacs-flavoured) lisp, and immediately evaluate it: `(message "Hello world")` ->  `M-x eval-buffer` and a "Hello world" message is logged to the \*messages\* buffer (a temporary file that's created when emacs is started for logging purposes). Emacs being a lisp interpreter is a very simple, yet powerful concept that enables extreme customizability and is why Emacs is one of the oldest editors in the programming world that has stood the test of time and is still used by many devoted fans today.

## What are my basic requirements for an editor?

When I start hacking on a project, whether I'm starting it from scratch or I'm jumping in to a large complex codebase, I have a few *must haves* that are non-negotiable if I'm going to be doing non-trivial work (i.e more that just updating a single file to submit a patch):

* Searching/finding - I need to be able to quickly find files and/or keywords/usages in a project. (All self-respecting editors can do this, but emacs packages help do it on steroids).
* Syntax highlighting and smart completion for languages that I work with. This is the most trivial box that pretty much any editor can tick for me.
* Quickly build and test my project/source files (this is where most other editors fall short for me).

With all that in mind let's get our first taste of emacs.

## Emacs terminology and basics for this tutorial (don't skip this!)
- Emacs commands are executed with `C-*`, `M-*` or `C-M-*`, "C" is the control key, and "M" is the _meta_ key which is not really a key on modern keyboards anymore, but maps to the "option" key on a Mac (later we'll switch this to the "Cmd or Command" key on Mac because I find that easier, but if you don't mind using the option key then skip that step!)
- Frame: what you normally call a "window" for GUI applications, basically it's one running instance of an emacs client. Each frame holds atleast one _window_
- Buffer: used to hold the contents of files that are being visited
- Window: the area of screen that displays a buffer
- Copying and pasting: in emacs this is called killing and yanking, respectively, for legacy reasons that aren't important here. To copy or _kill_ a highlighted area: `C-w` and pasting or _yanking_: `C-y`.
- If/when you get stuck in some command and you don't know how to get out of it, spam `C-g` (*This is probably the most important command in all of emacs*).

## TL;DR
If you don’t feel like going through this tutorial step by step and just want the code, you can find it at my [emacs-light repo](https://github.com/shaneikennedy/emacs-light.git). Follow the README there to get started.

## Let's start

> Disclaimer, this is going to be a MacOS walkthrough because most devs work on macs afaik because their company requires it, but if you're on linux 95% of this should be applicable and the other 5% should be easy enough to figure out. If you're on windows you can use the Windows Subsystem for Linux.

As you may have heard/seen, most people configure their emacs/vim with "dotfiles" that are loaded when you start emacs/vim. We'll get there, but first I'm going to show you why the idea that Emacs being a lisp interpreter is so powerful.

I'm going to assume you have emacs installed already (if you don't, [go do that](https://wikemacs.org/wiki/Installing_Emacs_on_OS_X)), so open a terminal in your home directory and run `emacs` which should open up a blank, bland window for you with a bunch of intro information (if this has errors open emacs from your Applications folder).

I'm going to hold your hand through the first few steps because raw emacs has a.... not so modern first-time-user-experience -- to put it lightly.

Press C-x and then C-f. You should then see the "_mini buffer_" at the bottom that's indicating that you are in the home "\~/" directory. This key-combination is mapped to `find-file`, an emacs function for finding files from your current directory (for us, this is the _home_ directory "\~"). Type .emacs and hit enter. This has just created a new buffer for a file named ".emacs" in your home directory.

![find-file](/images/find-file.png "find file")

The next thing I want you to do is press M-x and type emacs-lisp-mode and then press enter RET (enter). We've learned two things here:

* M-x is where you find all emacs functions
* The idea of modes - this is a big topic in and of itself, but for now it's important that each language needs a mode so that emacs can understand how to treat it (syntax highlighting, language specific refactoring, code execution etc.)

What we just did is make emacs think that this is an emacs-lisp (elisp) source file by enabling emacs-lisp-mode. You configure emacs by writing elisp, which it can then interpret and evaluate. What we're going to do is write elisp in this file ".emacs" and evaluate this file step by step so you can see how emacs changes at each step of the way.


Like I said before, we're about to configure emacs with elisp and so this is a configuration file; but we're not going to write every last bit of functionality from scratch! Like most text-editors/IDEs today, emacs has a package system, and this configuration file will consist primarily of declarations for which packages we want emacs to use.

Emacs packages are written in elisp and are hosted on various package repositories such as Milky Postmans E-Lisp Package Archive, or, [MELPA](https://melpa.org/).

Now let's get started!

By now you should be in a file called ".emacs" and have enabled `emacs-lisp-mode`. Lets start by configuring some package repository information:

``` lisp
(require 'package)

;; Nice macro for updating lists in place.
(defmacro append-to-list (target suffix)
  "Append SUFFIX to TARGET in place."
  `(setq ,target (append ,target ,suffix)))

;; Set up emacs package archives with 'package
(append-to-list package-archives
                '(("melpa" . "http://melpa.org/packages/") ;; Main package archive
                  ("melpa-stable" . "http://stable.melpa.org/packages/") ;; Some packages might only do stable releases?
                  ("org-elpa" . "https://orgmode.org/elpa/"))) ;; Org packages, I don't use org but seems like a harmless default

(package-initialize)

;; Ensure use-package is present. From here on out, all packages are loaded
;; with use-package, a macro for importing and installing packages. Also, refresh the package archive on load so we can pull the latest packages.
(unless (package-installed-p 'use-package)
  (package-refresh-contents)
  (package-install 'use-package))

(require 'use-package)
(setq
 use-package-always-ensure t ;; Makes sure to download new packages if they aren't already downloaded
 use-package-verbose t) ;; Package install logging. Packages break, it's nice to know why.

;; Slurp environment variables from the shell.
;; a.k.a. The Most Asked Question On r/emacs
(use-package exec-path-from-shell
  :config
  (exec-path-from-shell-initialize))
```


Ok all this code so that we now have access to `use-package`. From here on out, when we want to install and make a package available to emacs all we need to do is add `(use-package package-name)` which will pull the package source files from the repositories we enabled above: melpa, melpa-stable, and org-elpa, install them and load them into emacs.

Now lets evaluate this code: `M-x eval-buffer`

Nothing noticeable changes here because we just configured a package manager, but now lets install a theme from an emacs package, and enable it (because let's face it, the default in emacs is not very pretty).

``` lisp
(use-package doom-themes
  :init
  (load-theme 'doom-one))
```

Before when we configured the package manager there was a bunch of code and it was all new so it made sense to just go ahead and evaluate the whole buffer. But here we just added these few lines, why re-evaluate the whole thing? Put your cursor anywhere inside the (use-package...) parens, then hit `M-x eval-defun` (this function also has a keybinding C-M-x, remmeber this, we're constantly going to be evaluating elisp chunks like this).

You are probably going to get prompted with a yes/no for trusting this theme now and in the future (after all even themes in emacs are just elisp, so don't evaluate any elisp/themes you don't trust! But we trust the [doom maintainer](https://github.com/hlissner/emacs-doom-themes), so go ahead and type yes)


![Doom themes](/images/doom-theme.png "doom themes")


Tada! You just downloaded an emacs package of themes, and told emacs to load one of them and it works!

A few more UX importvements

``` lisp
;; Any Customize-based settings should live in custom.el, not here.
(setq custom-file "~/.emacs.d/custom.el") ;; Without this emacs will dump generated custom settings in this file. No bueno.
(load custom-file 'noerror)

;;; OS specific config
(defconst *is-a-mac* (eq system-type 'darwin))
(defconst *is-a-linux* (eq system-type 'gnu/linux))

;; Emacs feels like it's developed with linux in mind, here are some mac UX improvments
(when *is-a-mac*
  (setq mac-command-modifier 'meta)
  (setq mac-option-modifier 'none)
  (setq default-input-method "MacOSX"))

;; Some linux love, too
(when *is-a-linux*
  (setq x-super-keysym 'meta))

;; Fullscreen by default, as early as possible. This tiny window is not enough
(add-to-list 'default-frame-alist '(fullscreen . maximized))

```

One more `eval` trick: highlight the code you just pasted in and run `M-x eval-region`.

From here on out your Meta (M-*) key is now the command key (if you're on a Mac). I find this 100x better than the option key, but feel free to pick and choose from this block.

One final block of UX improvements that will be much appreciated, I promise.
``` lisp
;; Make M-x and other mini-buffers sortable, filterable
(use-package ivy
  :init
  (ivy-mode 1)
  (setq ivy-height 15
        ivy-use-virtual-buffers t
        ivy-use-selectable-prompt t))

(use-package counsel
  :after ivy
  :init
  (counsel-mode 1)
  :bind (:map ivy-minibuffer-map))
```
Highlight this region and do `M-x eval-region` again to let these changes take effect. Now hit M-x again and see that you now can see everything in emacs at your finger tips; from now on you'll be able to search and discover all of the emacs functions that you can interact with through M-x. Let's keep going.

![Meta X](/images/meta-x.png "Meta-x menu")

## Let's start with my first necessity in an editor: "_Searching/finding - I need to be able to quickly find files and/or keywords/usages in a project._"

For this we're going to use a mix of two packages: [projectile](https://github.com/bbatsov/projectile) and [counsel-projectile](https://github.com/ericdanan/counsel-projectile)
``` lisp
;; We need something to manage the various projects we work on
;; and for common functionality like project-wide searching, fuzzy file finding etc.
(use-package projectile
  :init
  (projectile-mode t) ;; Enable this immediately
  :config
  (setq projectile-enable-caching t ;; Much better performance on large projects
        projectile-completion-system 'ivy)) ;; Ideally the minibuffer should aways look similar

;; Counsel and projectile should work together.
(use-package counsel-projectile
  :init
  (counsel-projectile-mode))

```

Highlight the region, M-x eval-region and now if you check M-x projectile- you should have a bunch of possible commands for registering projects, switching to them, finding files and searching files.


Let's try an example `M-! git clone https://github.com/codemirror/CodeMirror.git ~/code-mirror`(Notice when you type M-! the mini buffer pops up with a shell command prompt? This is great for running quick commands). CodeMirror isn't tiny, this might take a few seconds...

Now lets register CodeMirror as a project `M-x projectile-add-known-project` and then find the path where you just cloned the repo to (~/code-mirror if you followed the above directly).

Now lets open the project `M-x projectile-switch-project` (Maybe open a new buffer in emacs first so you don't lose your config file, we're going to work in this project for a little bit `C-x 3` to open a window to the right). code-mirror should be the only project registered, so choose that and then you should be shown a list of files in the project that you can open; lets choose src/modes.js.

Projectile rapid fire:

- *How can I register another project?* `M-x projectile-add-known-project`
- *How can I switch to another project?* `M-x projectile-switch-project`
- *How can I search for other files in this project?* `M-x projectile-find-file`
- *How can I do find-and-replace in this project?* `M-x projectile-replace`
- *How can I search for strings/words in this project?* `M-x projectile-grep` or `M-x projectile-ag` or `M-x projectile-rg` (I recommend [rg](https://github.com/BurntSushi/ripgrep) but it's seperate from emacs so you'll need to install it. If you don't want to bother with that, use projectile-grep, but grep is noticeably slow on larger projects).
> This will open a buffer with a list of occurences which is nice, but counsel-grep/ag/rg will show you results in the minibuffer so that you can quickly see your search results without opening another window. It's excellent, I don't know of any other editor that has this.


> Bonus: Another gem in emacs comes from the ivy package we installed earlier for searching within a file `M-x swiper`


## On to my second necessity in an editor "_Syntax highlighting and smart completion for languages that I work with._"

Emacs comes with a bunch of built-in language modes for popular languages such as python, js etc. but let's configure completion capabilities.

``` lisp
;; Company is the best Emacs completion system.
(use-package company
  :bind (("C-." . company-complete))
  :custom
  (company-idle-delay 0) ;; I always want completion, give it to me asap
  (company-dabbrev-downcase nil "Don't downcase returned candidates.")
  (company-show-numbers t "Numbers are helpful.")
  (company-tooltip-limit 10 "The more the merrier.")
  :config
  (global-company-mode) ;; We want completion everywhere

  ;; use numbers 0-9 to select company completion candidates
  (let ((map company-active-map))
    (mapc (lambda (x) (define-key map (format "%d" x)
                        `(lambda () (interactive) (company-complete-number ,x))))
          (number-sequence 0 9))))

;; Flycheck is the newer version of flymake and is needed to make lsp-mode not freak out.
(use-package flycheck
  :config
  (add-hook 'prog-mode-hook 'flycheck-mode) ;; always lint my code
  (add-hook 'after-init-hook #'global-flycheck-mode))

;; Package for interacting with language servers
(use-package lsp-mode
  :commands lsp
  :config
  (setq lsp-prefer-flymake nil ;; Flymake is outdated
        lsp-headerline-breadcrumb-mode nil)) ;; I don't like the symbols on the header a-la-vscode, remove this if you like them.
```

Highlight, `M-x eval-region`, and try typing "(use-" in your current file and see the completion dropdown (if you're not seeing it, try C-. you won't have to do this every time, but sometimes I notice it doesn't initialize properly). This completion is the company package at work.

![Completion](/images/completion.png "completion")

Now for an example, lets make use of the CodeMirror repo that we cloned earlier. Open up src/modes.js in CodeMirror and Run `M-x lsp`. This will prompt you to choose a language server, choose ts-ls and it will begin downloading it for you. Once it's done, it will ask you where the project root is (~/code-mirror).

To make sure the language server is working, move your cursor to line 1 in modes.js on to `copyObj` and then `M-x lsp-go-to-implementation` and you should just to a file called "misc.js" where the function is defined. Great, so lsp is analyzing the codebase which means we have some _smart_ completion.


## And now for my final essential in an editor: _Quickly build and test my project/source files_

As I mentioned above this is where most editors tend to fall short for me; the reason being that it can be hard to just test one function at a time, it runs in some special test window which makes it _different_ to go debug, or it's just not fast enough to go from writing/modifying a test to executing it.

Since we're working on a javascript project, we're going to install a package called [npm.el](https://github.com/shaneikennedy/npm.el) which will provide some handy functions for quickly running npm commands.

``` lisp
(use-package npm)
```

Put the cursor inside the parens and `C-M-x` to evaluate that function.

Now back to the code mirror (reopen misc.js, or any file in the project if you don't still have that open) and run `M-x npm` and you'll see a menu pop-up on the bottom with a few possible npm options. We're going to start with npm install, choose "i" and when it asks for a package name leave it blank to simply run npm install to install everything in package.json. When that finished lets try "run", `M-x npm` and press "r" and you should get a list of options to choose from, choose one and hit Enter. You should see a new buffer open that's running the command you chose. This is called a compilation buffer and it's very useful when you are repeatedly running a certain command, like when using TDD or refactoring a module and frequently need to re-run tests. To re-run the command that you chose earlier, you don't need to go through the entire `M-x npm` flow again, you just need to switch to that buffer and hit "g"; this is a special command in "compilation-mode" which means "recompile".


And there you have it, lightning speed building and testing in javascript projects. But this type of flow isn't limited to javascript; there are packages for all sorts of languages, build tools and other technologies you're working on; just checkout [MELPA](https://melpa.org). And as a last resort, you can always `M-x compile` and write out the command you want manually to get a compilation buffer that you can `M-x recompile` as you need.


To save this file so that emacs starts with this configuration the next time you open it hit `C-x s`.

This is the bare minimum emacs configuration for what my basic needs in an editor are, but the emacs world is vast and exciting to explore. Follow some emacs-ers on github, checkout youtube, and read blog posts to see all kinds of interesting things you can do with emacs.

## Appendix

### Commands and keybindings

These are some of my most used commands, some of them are built in to emacs and others come from the packages installed in this tutorial. If they have a default keybinding then I've listed it below but if not I've left it as N/A

| Command               |      Keybinding        |  Description                                                                                                                   |
|-----------------------|------------------------|--------------------------------------------------------------------------------------------------------------------------------|
| swiper                | N/A                    | Search within the current buffer for a string/regex and results are dislplayed in the minibuffer                               |
| counsel-rg            | N/A                    | Search the entire current project for a string/regex using ripgrep and results are displayed in the minibuffer                 |
| projectfile-find-file | N/A                    | Search your current project for a filename                                                                                     |
| ivy-switch-buffer     |`C-x b`                 | Switch to another previously opened buffer                                                                                     |
| delete-window         |`C-x 0`                 | Delete the current window                                                                                                      |
| delete-other-windows  |`C-x 1`                 | When you have a bunch of windows this bufferes open at once and need to just delete them all except the one you're working on  |
| split-window-right    |`C-x 3`                 | Open a new window to the right                                                                                                 |
| save-buffer           |`C-x s`                 | Save the contents of the buffer you're working on                                                                              |
| comment-line          |`C-x C-;`               | Comment out the current line                                                                                                   |
| kill-region           |`C-w`                   | Copy highlighted text                                                                                                          |
| yank                  |`C-y`                   | Paste from clipboard                                                                                                           |


### Package recommendations
A great package for learning emacs is [which-key](https://github.com/justbur/emacs-which-key). This package makes a minibuffer popup with a key map based on the last key you just pressed (i.e if there are any). Add `(use-package which-key :config (which-key-mode t))` to your config and try pressing `C-x` to see what options you have from there.

[Magit](https://magit.vc/) is the best git client you can get. All other git clients should aspire to be how great Magit is.

[diff-hl](https://github.com/dgutov/diff-hl) Git diff markers in modified buffers. Nice to see which lines you've added, changed and deleted, visually.

[smart-parens](https://github.com/Fuco1/smartparens) bracket/parens matching is nice


### Doom emacs and Spacemacs

[Doom emacs](https://github.com/hlissner/doom-emacs) and [Spacemacs](https://www.spacemacs.org/) are "emacs distributions": when installed you get an entirely pre-configured emacs with all of the nice bells and whistles already there for you. I personally started with spacemacs and then moved to my own emacs config later. One massive caveat for spacemacs is that it is highly intergrated with the "evil" package, which means it uses vim keybindings. While you can disable "evil-mode", the configuration will be greatly hindered without it.
