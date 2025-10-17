---
title: "M-x Chill: Vibe Coding in Emacs with Amp"
date: "2025-06-07"
description: "No more venturing to foreign electron-based lands to catch a vibe"
---

DISCLAIMER: it's all vibe coding if you actually like coding 🎤

For this post and tutorial we're going to be using Sourcegraph's [amp](https://ampcode.com/) and [amp.el](https://github.com/shaneikennedy/amp.el) to turn emacs into the ultimate vibe coding machine.

You could probably whip up something similar with claude code and/or codex, but I had such a wow experience trying amp that I think I'm sticking with it for the foreseeable future. If you haven't tried it yet, I highly recommend you do.

## Requirements

- Emacs 28.1 or later
- Node.js and npm (for Amp CLI installation)
- Optional: `projectile` or `project.el` for enhanced project detection

## Installation

### Using straight.el

Add this to your Emacs configuration:

```elisp
(straight-use-package
  '(amp :type git :host github :repo "shaneikennedy/amp.el"))
```

### Manual Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/shaneikennedy/amp.el.git
   ```

2. Add to your Emacs `load-path`:
   ```elisp
   (add-to-list 'load-path "/path/to/amp.el")
   (require 'amp)
   ```

And that's it.

Now open a project in emacs, or navigate to any file in a project and just run `M-x amp` to start a session. If you don't have amp installed amp.el will try to install it for you, and you will need to login to your amp account.

Once that's done you're ready to start vibe coding. Type your first prompt and watch it go to work.

Whatever your opinion is on the future of software engineering with AI is, this shit is so fun. Idea to prototype in minutes. If you have a vision and know how to do it anyways this just accelerates the prototype to finished-project phase, and if you don't have a vision or know how to get there enjoy the ride and learn. Use it to augment your own skills or let it try to show you the way.

But most importantly, **we're back in emacs**. No more being a fish out of water in vscode or one of it's various forks to ride the AI wave; everything is a buffer again, amp.el uses native emacs stuff we're all used to which means stuff should just work as it would with anyother term.el process, we get `<command>-for-region` style actions, life is good again.

Some really nice features to point out:

- `M-x amp` is _project_-based. You'll notce that when you start your first amp session the buffer is called \*amp-<project-name>\*, that means you can have multiple projects open at once in emacs, vibe coding in all of them. The amp.el commands are project aware thanks to project.el built in to emacs (🙏) so if you're using amp--fix-* in one project you don't confuse/mangle/mix the context from one project to another. I have 3 amp processes running while writing this.

- `M-x amp--prompt` and `M-x amp--prompt-for-region` send a generic prompt from anywhere in your project to amp or send it with some highlighted context

- `M-x amp--fix-region`: imagine you're wrtting some tests, `M-x rust-test` runs a compilation buffer and we get an assertion error. Navigate to that buffer, highlight the asserttion error, `M-x amp--fix-region` and bam, it gets right to work fixing that test.


- `M-x amp--improve-region` highlight some text, `M-x amp--improve-region` and bam, watch the buffer update your code as amp gets to work. I personally use this all the time when I'm working in a language where I'm not sure what the idiomatic way to express something might be, it's great for learning.

- `M-x amp--explain-region` Highlight some text and ask to explain it, pretty standard.

- `M-x amp--switch` shortcut to list all your open amp buffers and easily switch between them.

- `M-x amp--kill` shortcut to select a running amp buffer to kill.


## Other resources:
- [aideremacs](https://github.com/MatthewZMD/aidermacs) Really well done, bring-your-own-key so it's agent agnostic. It's also way more developed in terms of number of commands, and comes with a transient menu for discoverabillity. Definitely worth checking out if you don't mind the BYOK approach.
- [ellama](https://github.com/s-kostyaev/ellama) Also nice, similar BYOK, this is less "agentic" though, you won't get a similar experience here, ellama is more of a LLM integration to emacs. You can get it to do completion and ask questions etc but it won't go update your whole project for you if you ask it to.
