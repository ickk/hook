Hook
====

Hook is a simple webserver intended to respond to webhook events.


Currently Hook has a Git module that will respond to GitHub Webhook *push*
events. One use-case is to host the source code of a website in a git
repository and use Hook to allow easily updating the live website by pushing
changes to the repository.


Other kinds of webhooks responses should be straight-forward to add without
changing the overall architecture of Hook.

<!-- START doctoc generated TOC please keep comment here to allow auto update -->
<!-- DON'T EDIT THIS SECTION, INSTEAD RE-RUN doctoc TO UPDATE -->
**Table of Contents**

  - [Configuration](#configuration)
- [Modules](#modules)
  - [Git](#git)
    - [Configuration](#configuration-1)
- [Build Instructions](#build-instructions)
- [Architecture](#architecture)
  - [Git](#git-1)

<!-- END doctoc generated TOC please keep comment here to allow auto update -->

### Configuration ###

Hook reads all it's configuration values from
[RON](https://github.com/ron-rs/ron) formatted configuration files found in a
`config/` folder relative to the current working directory when running the
Hook binary: `$PWD/config/*.ron`.


Hook itself takes some configuration options:


`config/hook.ron`:
```ron
(
  address: "localhost",
  port: 7267,
  url_base: "/hook",
  modules: [
    Git (mount_path: "/git"),
  ],
)
```

* `address` and `port` are the interface to serve Hook at.
* `url_base` should be set if Hook is being served at a path by a reverse proxy
  instead of from the site root. The base path will be prepended to any
  internally generated urls to correct for the reverse proxy.
* `modules` takes a list of Hook modules to enable, and each module takes a
  `mount_path` that it will be served under.


Configuration of individual modules is covered in each [Modules](#modules)
subsection.


Modules
-------

### Git ###

The Git module will respond to [Github webhook events](https://docs.github.com/en/developers/webhooks-and-events/about-webhooks>`).


The *push* event is currently supported with a corresponding *pull* action,
allowing Hook to automatically pull changes into a git repository when new
changes are available on the remote. Basic validation of the event is done;
HTTP headers are checked, the HMAC signature of the event is verified.


To use the Git module, it must first be enabled in the `config/hook.ron`
configuration file:


#### Configuration ####

`config/hook.ron`:
```ron
(
  modules: [
    Git (mount_path: "/git"),
  ],
)
```


Then the Git module can be configured with the `config/git.ron` configuration
file:


`config/git.ron`:
```ron
(
  policies: [
    ( service: Github,
      repo_name: "user_a/repository_1",
      secret: "super_secret_74830921478320147830214783021478187",
      event: Push,
      action: Pull (path: "/home/user_a/repository_1",
                    remote: "origin",
                    branch: "main",
                    ssh_key_path: "config/repository_1.id_rsa") ),
    ( service: Github,
      repo_name: "user_b/repository_2",
      secret: "meta_secret_748930216874830214316248923017483201",
      event: Push,
      action: Pull (path: "/home/user_b/repository_2",
                    remote: "origin",
                    branch: "prod",
                    ssh_key_path: "config/repository_2.id_rsa") ),
  ],
)
```


`policies` is a list of Policy definitions for which events to respond to and
  what actions to take. For each Policy:

* `service` takes the `Github` class.
* `repo_name` defines the *full repository name*. i.e. the
  user_name/repository_name.
* `secret` is the shared secret that is used to cryptographically sign
  webhooks. It should be a long random string of characters.
* `event` is the type of event to respond to. It takes the `Push` class to
  respond to Github *push* webhook events.
* `action` is the type of action to take in response to the event. The `action`
  field takes a `Pull` class with it's own fields:
  * `path`: The path of the git repository on the local disk.
  * `remote`,`branch`: The git *remote* and *branch* to pull.
  * `ssh_key_path`: The path to an ssh private key file authorized to pull from
    the specified remote.


Note: On Github you can add *read-only* ssh keys from the *Deploy keys* section
on a repository's settings page. It is recommended to use a Deploy key with
unattended services such as Hook.


Build Instructions
------------------

Dependencies:
* [Rust/Cargo](https://rustup.rs),
* GNUMake.

Build with:
```sh
make build
```

Run with:
```sh
./hook
```
Note: First setup [configuration](#configuration) files.


Architecture
------------

Hook is a [Rocket](https://rocket.rs/)-based web application. Rust modules form
collections of end-points related to a single service. Each module provides a
`pub fn routes() -> Vec<Route>` implementation that returns all of the relevant
Rocket routes.


A selection of modules are enabled in the `hook.ron` configuration file and
then `main.rs` mounts the specified modules to Hook's request router.


### Git ###

The service and event type are pulled from the request's HTTP headers (In this
context *service* refers to Github, Gitlab, etc. and *event* refers to pull,
push, issue, etc.)

Multiple request handlers may implement the `/<repo_name>` end-point, with each
handler targeting a different service/event combination to handle their
service-specific quirks.
