---
title: Configuring emacs from scratch in 2025
date: "2025-06-14"
description: "How I rewrote my 5 year old emacs config from scratch. Modern packages from the community, making use of all the recent improvements in emacs-proper, and of course vibecoding."
---

A few years ago I wrote my love letter to emacs, a blog post I called ["Your first taste of emacs"](/blog/emacs-intro) about why I love emacs, what I think it does better than most other editors, how I use it and what I think are the absolute basics to getting started with emacs. For the love letter parts, go read that post, but the configuration I shared in that post could absolutely use some updating; emacs and the community have made some great progress in recent version of emacs, and well, LLMs happened since that post.

This config focuses on a few main things:
1. I try to use new and actively maintained packages. The ones from [minad](https://github.com/minad) on github particularly interested me because they focus on recent versions of emacs and "doing one thing well".
2. Using more of the emacs builtins! A lot of great packages have been merged into emacs in the last few years, they're well polished, maintained and do the work of third-party packages I used to rely on.
3. Tree sitter, I still don't have the hang of structural editing that tree-sitter enables but even just the improved syntax highlighting makes it worth setting up.
4. It is impossible not to talk about AI and the importance of having tooling integrated into your editor, no matter what your stance is on the future of coding it's just useful to have these tools at your fingertips with a slick `C-*` keybinding.


Let's get package management out of the way, we are going to use some third-party packages and I want to be able to pull them automatically (i.e I don't want to vendor them in), I like [straight.el](https://github.com/radian-software/straight.el) because you can pull from github when a package isn't on [melpa](https://melpa.org), but in general I like pulling from melpa and for that I use `use-package`.

```emacs-lisp
(defvar bootstrap-version)
(let ((bootstrap-file
       (expand-file-name
        "straight/repos/straight.el/bootstrap.el"
        (or (bound-and-true-p straight-base-dir)
            user-emacs-directory)))
      (bootstrap-version 7))
  (unless (file-exists-p bootstrap-file)
    (with-current-buffer
        (url-retrieve-synchronously
         "https://raw.githubusercontent.com/radian-software/straight.el/develop/install.el"
         'silent 'inhibit-cookies)
      (goto-char (point-max))
      (eval-print-last-sexp)))
  (load bootstrap-file nil 'nomessage))

(setq package-enable-at-startup nil)
(straight-use-package 'use-package)

;; Configure package archives
(setq package-archives
      '(("melpa" . "https://melpa.org/packages/")
        ("gnu" . "https://elpa.gnu.org/packages/")
        ("nongnu" . "https://elpa.nongnu.org/packages/")))

(setq use-package-always-ensure t)
(setq straight-use-package-by-default t)
```

Now we can do `(use-package <package-name>)` for a package on melpa and `(straight-use-package '(<package-name> :type git :host github :repo "<repo-owner>/<repo-name>"))` for installing right from source.

I always have a bunch of "quality of life" default settings, I won't write them all down here but you can find them on my config [shaneikennedy/.emacs.d](https://github.com/shaneikennedy/.emacs.d). I'm also not going to go through stuff like magit or my keybindings for moving around emacs, you can see them in my config if you care or read the "first taste of emacs" post above if you care but I'm going to focus on the 2025 stuff.

Let's get to configuring emacs completions, I make heavy use of [minad](https://github.com/minad)'s packages here:
- [vertico](https://github.com/minad/vertico)
- [corfu](https://github.com/minad/corfu)
- [consult](https://github.com/minad/consult)
- [marginalia](https://github.com/minad/marginalia)
- [orderless](https://github.com/oantolin/orderless) (this one is from [oantolin](https://github.com/oantolin))

```emacs-lisp
;; Vertico for vertical completion for the emacs minibuffer.
(use-package vertico
  :init
  (vertico-mode))

;; Emacs minibuffer configurations.
(use-package emacs
  :custom
  ;; Hide commands in M-x which do not work in the current mode.  Vertico
  ;; commands are hidden in normal buffers. This setting is useful beyond
  ;; Vertico.
  (read-extended-command-predicate #'command-completion-default-include-p))

;; A bunch of great search and navigation commands
(use-package consult
   :hook (completion-list-mode . consult-preview-at-point-mode)
   :custom
   (consult-preview-key nil)
   (consult-narrow-key nil)
   :config
   (consult-customize consult-theme consult-line consult-line-at-point :preview-key '(:debounce 0.2 any))
 )

;; Annotations in the minibuffer, i.e a description of the function next to the name in M-x
(use-package marginalia
  :init
  (marginalia-mode))

;; In buffer completions, think lsp completions
(use-package corfu
  :custom
  (corfu-auto t)
  (corfu-cycle t) ;; Enable cycling for `corfu-next/previous'
  :bind
  (:map corfu-map
        ("TAB" . corfu-next)
        ("C-n" . corfu-next)
        ([tab] . corfu-next)
        ("C-p" . corfu-previous)
        ("S-TAB" . corfu-previous)
        ([backtab] . corfu-previous))
  :init
  (global-corfu-mode))

;; Completion style and fuzzy matching
(use-package orderless
  :custom
  (completion-styles '(orderless basic))
  (completion-category-defaults nil)
  (completion-category-overrides '((file (styles partial-completion)))))
```

With just this little bit of code the M-x experience is top-quality (fuzzy searching, docstrings, narrowing), later when we hook up the lsp you'll have the completion experience you'd expect from an editor in 2025, you have access to search tools from consult like consult-line to search text in a buffer and consult-find (consult-fd iykyk) for searching for files in a project and consult-ripgrep for searching text in the entire project. And please immediately use consult-theme to choose a better theme than the emacs default, `(use-package doom-themes)` if you don't know of any.


![M-x](/images/M-x.png)
![consult-line](/images/consult-line.png)
![consult-theme](/images/consult-theme.png)


Now that we have a nicely functioning emacs at this point (see the difference of stock emacs M-x vs now that we have some of these packages loaded) let's move on to what I've swapped out from my old config for the "new" builtins in the recent versions of emacs:

- lsp-mode -> eglot
- projectile -> project.el

Moving from lsp-mode to eglot has been a major, major upgrade for me. It's much more minimal than lsp-mode but that's what I want from my lsp client, I think lsp-mode tries to mimic the vscode aesthetic and UX which just isn't my cup of tea.

projectile -> project honestly I don't think I've noticed a difference, the commands that I use feel identical to me, I never had any problems with projectile and I don't have any with project.el. The only reason I switched was to have one less package to pull from melpa.


Now on to tree-sitter. Like I said, I still haven't gotten the hang of structural editing yet in my workflow, I'm just too used to and too fast with a grep flow that switching my mental model from grep to "I want to move up a node in the hierarchy of this function" (?) just doesn't compute for me yet. Idk skill-issue or whatever, I'm trying, but regardless if that clicks with you or not the syntax highlighting that you get in treesitter modes is just better and worth setting up.

```emacs-lisp
(use-package treesit-auto
  :custom
  (treesit-auto-install 'prompt) ;; if a treesitter grammar can't be found for the language detected in the buffer, prompt me to install it
  :config
  (treesit-auto-add-to-auto-mode-alist 'all) ;; if a treesitter grammar is found for the language detected in the buffer, use the corresponding language-ts-mode
  (global-treesit-auto-mode))
```

It's this easy, massive shoutout to [renzmann](https://github.com/renzmann/treesit-auto) because I tried getting this set up before finding this package and it's a mess. Manually managing tree-sitter grammars, manually remapping all <language>-mode to <language>-ts-mode, no thanks, this really ought to be fixed in a new version of emacs.


Now the fun part, LLMs and vibecoding, how can we talk about an editor in 2025 that isn't "agentic" or have a "copilot"?

I use a few tools here:
- [ellama](https://github.com/s-kostyaev/ellama) Basic LLM integration here to just be able to talk to the model, ask it questions etc. It also has some code actions like "complete" or "edit" but I haven't found great success with them. You can hook into [ollama](https://ollama.com/) here or provide an api key if you have one.
- [copilot](https://github.com/copilot-emacs/copilot.el), I'm going to be honest I have this one turned off most of the time but it's great to be able to turn it on when you want it. If I have to write golang, the constant `if err != nil` and returning a wrapped error with an error string it's undeniably nice to just tab complete that.
- [amp.el](https://github.com/shaneikennedy/amp.el) Disclaimer, I wrote amp.el. Amp is like claude code but from Sourcegraph, it's great and amp.el is a little emacs wrapper around term-mode that runs amp, manages buffers and has some quick functions for interacting with the amp buffer from anywhere in your project. I wrote about it more in depth here: [M-x Chill: Vibe Coding in Emacs with Amp](/blog/vibecoding-in-emacs-with-amp) if you're interested.

And that's it! After that it's up to you to make it your own, but I think if I was configuring my emacs from scratch again in 2025 I would use this as a jumping off point.

Thanks for reading,

Shane
