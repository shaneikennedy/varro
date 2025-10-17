---
title: First 10 things to do at your new job as a junior dev
date: "2022-04-28"
description: "My best tips for onboarding as a junior software engineer (or any level, really)."
---

When starting a new job (from junior to senior) here are the first 10 things I like to do related to the code/project you're going to be working on.

There are a bunch of things you can do outside of the context of code, like starting to build your network at the company, setting yourself up for productive 1:1s with your manager and getting to know your team better that I won't cover in this post, but are all equally important, so don't forget these!

I'm also going to skip the "get your dev machine setup" suggestion (at least the most basic configuration of it). I assume this is a given if you're a developer: download your editor/IDE of choice, a terminal app if you want one, and any CLI tools that make you more productive in your normal workflows. If you need any suggestions, checkout [Upgrade your command line](/command-line-upgrade).

I'm also going to assume you have an "onboarding buddy", someone who's responsible for making sure you're getting up to speed and someone you can ask a shameful amout of questions to (there's no shame in this, actually, but 3 jobs later and I still feel like I'm being annoying when I ask too many questions, even if I tell people that I onboard not to feel this way towards me). If you don't have an onboarding buddy assigned to you when you start, talk to your manager and _get one_; I promise you nothing will make your first 6 weeks at a company better than a good onboarding buddy.

In any of the following points you find yourself asking "how do I find this out?", ask your onboarding buddy. This is what they are there for.

### 1. Get a list of repositories you're going to need

Typically when you join a team you're responsible for a certain service, or area of the code/platform/product, so find out where your code lives and any dependecies it has to other repositories in your organization, and clone them to your local machine. When you finally start working in the code you're going to want to jump around alot trying to figure out how this code all works, and having everything in place will save you a bunch of time.

> Each of these projects probably also have a very basic README.md to get each of them running, be sure to go through these as well.

### 2. Run the product/platform/service locally

This may sound obvious, but you would be surprised how many times I hear that people don't run their service locally, and that they rely on tests to verify their changes. While this might work in some cases, there's nothing that gives your more confidence that your changes work than to use the product/platform/service the way your users will. Maybe you won't always need to run your service locally, but someday you will and you don't want to be 6 months in to a job not knowing how to do this, because now you can't ask your onboarding buddy (of course you can, just not as shamelessly as you could 6 months ago).

### 3. Run the test suite

When you first jump into the code it's going to seem like a lot (because it normally is), and you're not going to be able to grasp it all at once. One thing that helps me onboard quickly is to find a module/package/class/function that I'm interested in, and find its test suite. Tests should be dumb and easy to understand: what's being tested, what the test conditions are and what the expectations of the test are. Tests are great for learning the codebase

Additionally, when you finally start coding, you're still not going to have in-depth knowledge of the codebase, product or platform, so tests are really all you have to make sure you're not breaking anything, so make sure you can run your tests locally!

### 4. Ask your team what tools they like to use

This is an easy but often overlooked tip: this will show you who likes similar tools to you and show you some new ones. Ask for any command line aliases or functions, too!

### 5. Find the "Getting started" guide for your team

This might exist as the project's README.md file, in a Confluence page, or any number of places really. If there isn't one already, consider taking notes on your onboarding experience: where to start, what wasn't completely clear, and what made no sense at all. Once you've onboarded, summarize this in a document and start your team's "Getting started" guide.

### 6. Check out your project's CI/CD

When it's finally time to open your first PR it will hopefully have to go through a bunch of automatic checks: tests, linting, etc. Figure out what these steps are and make sure you can setup some local tooling to avoid having to re-push little fixes like styling and commit message format.

### 7. Ask if there are any git conventions within the team or organization

Some of these conventions might be enforce in CI checks while others might just be agreed upon within the team as "best practices". Either way, find out what they are and learn to incorporate them into your workflow.

### 8. Note down some quick wins that you can get for yourself

One of the best parts of being new is that you're a pair of fresh eyes: you see problems that others don't want to, don't care to, or just don't see at all. Take note of where you can immdeiately start to add value, no matter how small. This not only adds value to the team, but demonstrates good initiative and some excitement from your side, both excellent qualities for your manager and peers to see.

### 9. Understand the deploy process

You just merged your first change into master/main.... what now???? Figure out if you have any staging environments, how often changes are pushed to production, and how/when you can verify them.

### 10. Observe your system

Find out what your team uses for your product/platform/services observability. Understand the load this system is under, how it's performing and what sort of indicators you can look for _when_ your system starts to break.

> This has ties to #8 and #9, in my experience the observability is always one of the least maintained parts of the product because its development is often reactive rather than proactive (aka it only gets updated _after_ something goes wrong, and is **ripe** with quick wins). It's also where you're going to want to keep an eye on after you merge PRs (certainly after your first) to know if your change is performing under load in production.
