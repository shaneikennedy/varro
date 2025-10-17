---
title: Docker series - tips and tricks
date: "2021-03-27"
description: "Now that you've learned how to build and run containers, it's time to get to know the ecosystem and learn some tips and tricks for working with docker. This covers using pre-built images, mutli staged builds, the dockerignore file and some extra commands from the docker CLI."
---

## Pre-built images

In our introductory example we looked at a simple python application. For this, we built a docker image based off of [Ubuntu]() and then added a `RUN` statement to install our python interpreters. This worked well for our example, but lets examine things closer:

`docker run -it ubuntu:18.04`

``` shell
learnyouadocker git/master
❯ docker run -it ubuntu:18.04
root@6d1594013cbd:/# apt-get update
Get:1 http://archive.ubuntu.com/ubuntu bionic InRelease [242 kB]
Get:2 http://security.ubuntu.com/ubuntu bionic-security InRelease [88.7 kB]
...
root@6d1594013cbd:/# apt-get install python3
Reading package lists... Done
Building dependency tree
Reading state information... Done
The following additional packages will be installed:
  file libexpat1 libmagic-mgc libmagic1 libmpdec2 libpython3-stdlib libpython3.6-minimal
  libpython3.6-stdlib libreadline7 libsqlite3-0 libssl1.1 mime-support python3-minimal python3.6
  python3.6-minimal readline-common xz-utils
...
root@6d1594013cbd:/# python3 --version
Python 3.6.9
```
The default python version installed from `apt` in Ubuntu 18.04 is python 3.6.*, but what if we need python 3.7 or 3.8/9/10? The problem of needing a specific version of a program is consistent accross all languages, ruby, java, javascript etc. and each language has a different way of solving it. Python has deadsnakes, Javascript/Node uses `curl` commands to fetch certain versions from [https://nodejs.org/en/](https://nodejs.org/en/) and so on.

Rather than going through all of the trouble of installing specific versions, DockerHub has lots of pre-built images for most languages and all of their versions with the ability to select which Operating System you prefer you app to run in.

So for the sake of our example, let's look at how we could use a pre-built python image.

``` dockerfile
FROM python:3.6-stretch

COPY main.py .

CMD ["python", "main.py"]
```

> "Stretch" is a specific release of the [Debian](https://www.debian.org/) distribution of Linux.

This is nice because now we can focus on what matters to us: building and running our code. These types of pre-built images exist for lots of languages, just head over to [dockerhub](https://hub.docker.com) and seach for what language you're building in.


## Alpine

I made a quick reference to the alpine image in my first tutorial, particularly about it's very small footprint (just 5MB!). Alpine is a great way to reduce the size of your image which can help when moving your image over the wire: the smaller your image, the quicker you can `push` and `pull`. This is useful especially in Continous Integration (CI) pipelines that use docker to create reproduceable builds. Often times our CI pipelines have multiple steps and all need to download our image, the smaller your image the faster this happens.

Let's compare our intro example image with an alpine based image:

``` dockerfile
FROM python:3.6-stretch

COPY main.py .

CMD ["python", "main.py"]
```

``` dockerfile
FROM python:3.6-alpine

COPY main.py .

CMD ["python", "main.py"]

```

``` shell
REPOSITORY      TAG      IMAGE ID       CREATED         SIZE
my-app-deb      latest   164dc24ba635   2 minutes ago   901MB
my-app-alpine   latest   7d0f2ecf5f38   4 minutes ago   40.7MB
```

Using alpine for our app gets us an image that's ~20x smaller


## Multi-staged builds

Often times, especially in compiled languages, we have lots of build-time system dependencies that are not necessary to _run_ our app. Take a Go app for example: in order to build your binary you need the go cli, but running your app is as simple as invoking `./my-binary`. In the pursuit of smaller images we should get rid of the now-useless go cli by adding `RUN rm -rf /path/to/go` at the end of your dockerfile right before CMD. But what if you have 10, 20 or 100 build-time system dependencies?

To solve this docker has implemented Mutli-staged builds (MSBs). Let's go through an example by taking a look at the OSS project [lazydocker](https://github.com/jesseduffield/lazydocker).

Adding a simple Dockerfile at the root of this project:

``` dockerfile
FROM golang:alpine

WORKDIR lazydocker

COPY . .

RUN go build
```
And running `docker build -t lazydocker .` and then `docker run -it lazydocker ls` we see

``` shell
❯ docker run -it lazydocker ls
docker-compose.yml  go.mod              hooks               main.go             scripts
getter.rb           go.sum              lazydocker          pkg                 vendor
```
which contains our binary file `lazydocker`.

Good, now lets declare this part of the Dockerfile the "builder" stage, and lets add an "app" stage.

``` dockerfile
FROM golang:alpine as builder

WORKDIR lazydocker

COPY . .

RUN go build

FROM alpine as app

WORKDIR lazydocker

COPY --from=builder /go/lazydocker/lazydocker .

CMD ["./lazydocker"]
```

Breaking it down:
`FROM golang:alpine as builder` this doesn't change the way `FROM` works, it's just giving it a reference name so that we can refer to this part of the build later.

`FROM alpine as app` When the docker parser encounters a new `FROM` statement, it effectively starts from scratch, creating an entirely new build.

`COPY --from=builder /go/lazydocker/lazydocker .` using `--from` here changes the standard behavior of `COPY`. Normally COPY will look on your machine for the path that follows, but --from=builder tells docker that you're refering to the previous stage in this build that we named "builder"

Building this image and running `ls` now shows us a directory "lazydocker" with a single binary file in it (also called "lazydocker")

``` shell
❯ docker run -it lazydocker ls
lazydocker
```
Let's look at the size difference

``` shell
REPOSITORY                TAG       SIZE
lazydocker-multi-stage    latest    18.4MB
lazydocker-single-stage   latest    406MB
```

With MSBs we achieve a build that is more than 20x smaller than if we did no cleaning up of build-time dependencies, and it's done in a way thats easy to understand and that looks 100000x better than a bunch of rm -rf commands.


## .dockerignore

Similar to a .gitignore to make sure you don't commit files you don't need into git history, we have the concept of a dockerignore file to make sure docker does not consider certain files when building images. This is important for 2 important reasons
1. Managing docker [context](https://docs.docker.com/engine/reference/commandline/build/#extended-description) size
2. Making sure you aren't copying certain files into your image when you declare a `COPY` directive in your Dockerfile

##### More on "context"
Remember when we broke down the build command in the first tutorial? `docker build -t my-app .` and we said that the `.` here was the PATH? What we're telling docker here is that this build command should expect to be able to run `COPY` from anything in the directory (or subdirectories) of `.`.

Docker prepares the build by "sending" the context to the docker daemon (the program that actually builds your images):

``` shell
❯ docker build -t my-app .
[+] Building 0.4s (9/9) FINISHED
 => [internal] load build definition from Dockerfile              0.3s
 => => transferring dockerfile: 37B                               0.1s
 => [internal] load .dockerignore                                 0.3s
 => => transferring context: 2B                                   0.1s
 => [internal] load metadata for docker.io/library/ubuntu:18.04   0.0s
 => [1/4] FROM docker.io/library/ubuntu:18.04                     0.0s
 => [internal] load build context                                 0.0s
 => => transferring context: 28B                                  0.0s
```
Look specifically of the last two lines of the output "load build context" and "transferring context: 28B". My example project is very simple with just a main.py file and a Dockerfile, so the build context is very small and this step happens in "0s". But lets consider a slightly more complicated example with nodejs. We're going to be using the [vue-cli](https://github.com/vuejs/vue-cli) template app as an example here:

``` shell
docker-ignore git/master
❯ ll
babel.config.js
node_modules
package-lock.json
package.json
public
README.md
src
```

Running `vue create docker-ignore` generates the project shown above and runs `npm install` to install the necessary dependencies to run the app, as defined in "package.json". Lets say we wanted to build a production image for this project... we would write a Dockerfile like the one below:

``` Dockerfile
FROM node:10-alpine

COPY package.json package-lock.json ./

RUN npm install --production

COPY . .

RUN npm run build

# Probably transfer built static files to an nginx container... see the Multi Staged Builds section
```

Lets build this
``` shell
❯ docker build -t docker-ignore .
 => [internal] load build context     1.4s
 => => transferring context: 1.71MB   1.3s
```

Now, this is still a pretty small project, so 1.71MB taking 1.3s to transfer might now be that big of a deal to you, but this number will grow significantly as you work on larger projects. Lets introduce a ".dockerignore" file to the root of the project with the following contents

``` text
node_modules
```

And then rebuild

``` shell
❯ docker build -t docker-ignore .
 => [internal] load build context     0.0s
 => => transferring context: 4.66kB   0.0s
```

This is a big difference and can really streamline your builds. If you're working on a project with docker and notice that your build context looks suspiciously large, take a look at your dockerignore file to see if something might be missing and add it, or add a dockerignore file to the project if there isn't one already.

On top of speeding up your builds, a dockerignore file can be useful to make sure you aren't copying over files that you don't need in the first place (and then having to clean them up in the dockerfile). Looking at the example above, we probably don't need to ship the README file to production! We can avoid this by adding a `RUN rm -rf README.md` to our dockerfile (ugly, bad), or we can simply add "README.md" to the .dockerignore file to make sure it never gets copied over in the first place.


## Get to know the docker cli

* Need to see what images you have locally? `docker images`
* Need to see which containers are running? `docker ps`
* Need to see how many layers your image has? `docker history <image_tag> | wc -l`
* Need to stop a running container? `docker stop <container_name>`*Note that container name is different than tag name: container name is assigned on docker run, while tag name is assigned on docker build.*

## Editor integrations
* VSCode: [docker plugin](https://code.visualstudio.com/docs/containers/overview)
* Shell [lazy docker](https://code.visualstudio.com/docs/containers/overview)
* Emacs [docker.el](https://github.com/Silex/docker.el)
