# Contributing to touchHLE

Please also read the [code of conduct](CODE_OF_CONDUCT.md).

## Issues

Please bear in mind that there are infinitely many apps that do not work in touchHLE right now, so please don't open issues about apps that aren't in [the supported list](APP_SUPPORT.md), unless you've got it partially working (e.g. loaded up to the menu but the main game doesn't work). The fact that an app's splash screen (Default.png) shows up doesn't mean it's partially working.

## Source control and review

### Setting up the repo

touchHLE uses git source control. You can get the source code from GitHub like this:

```
$ git clone https://github.com/hikari-no-yume/touchHLE.git
$ cd touchHLE
```

touchHLE uses Gerrit for code review. [The touchHLE repo on GerritHub](https://review.gerrithub.io/q/project:hikari-no-yume/touchHLE+status:open) is where you can submit patches.

Log into GerritHub with your GitHub account. If you happen to have a Server Side Error, try again in a couple of hours as GerritHub server sometimes is a bit unstable.

If it is your first time using Gerrit, you will need to set your public SSH key on [Gerrit settings page](https://review.gerrithub.io/settings/#SSHKeys), as well as make sure your email matches the email set on the settings page (i.e. if you use those GitHub noreply emails you will need to set your real email).

After setting your key, make sure you can connect to Gerrithub with (replace `your-github-username-here` with your username):

```
$ ssh your-github-username-here@review.gerrithub.io -p 29418
```

You can then add GerritHub as a “remote” (again, replace `your-github-username-here` with your username):

```
$ git remote add gerrit "ssh://your-github-username-here@review.gerrithub.io:29418/hikari-no-yume/touchHLE"
```

Gerrit requires your commit messages to have a `Change-Id:` line in them. Gerrit provides a git hook that adds this line to your commit messages automatically. You can install it like this (Windows users may need to use git bash for this):

```
$ (f=`git rev-parse --git-dir`/hooks/commit-msg ; mkdir -p $(dirname $f) ; curl -Lo $f https://review.gerrithub.io/tools/hooks/commit-msg ; chmod +x $f)
```

### Submitting changes

Make a local branch based on `trunk` with your changes. Try to avoid bundling unrelated changes in one commit. If you make a mistake in a commit that hasn't been merged yet, please fix it by modifying the original commit (e.g. using `git commit --amend --reset-author`), rather than by adding a separate commit.

Once you're happy with your changes, you can push them for review on Gerrit with:

```
$ git push gerrit HEAD:refs/for/trunk
```

Then go to GerritHub, make sure you didn't push anything you didn't intend to, and add `hikari_no_yume` as a reviewer.

Finally, add a +1 on Verified when you are sure your change is not breaking anything unrelated.

If you're submitting a large number of changes with a common theme, e.g. improving compatibility with some app, it is recommended to _also_ create a GitHub pull request. This improves visibility and ensures your changes are tested by the GitHub CI. You can then tag the Gerrit reviews with a “topic” named like `touchHLE/pull/xxx` where xxx is the pull request number. You can bulk-tag things using the checkboxes on the GerritHub homepage.

Please also see the following guidelines for what to do with code changes.

## Code contributions

[BUILDING.md](BUILDING.md) and [DEBUGGING.md](DEBUGGING.md) might be helpful while you're working on a contribution.

Please run `cargo fmt` and `cargo clippy` on your changes before committing. For the handful of C and C++ files, please use `clang-format -i` to format them.

You should also run `cargo test`. [Building the integration tests requires downloading LLVM](tests/BUILDING.md), so it's understandable if you want to skip them (`cargo test -- --skip run_test_app`) and let the GitHub Actions CI catch any issues when you submit your pull request. Alternatively, you can download a pre-built version of the integration tests app (TestApp) from GitHub Actions CI and run it in touchHLE.

If you're going to open a pull request with non-trivial changes, please talk to us first so we can figure out if we're likely to accept them. It would be a shame if your effort had to be wasted.

### Copyright and reverse engineering

(Please also read the copyright rules in the code of conduct.)

⚠️ Be **very** careful about copyright. To put it simply: **don't contribute if you've seen code you shouldn't have seen, don't copy code that isn't yours to copy, and especially don't _secretly_ copy and pretend you didn't**. Any infringement of Apple or other copyrights could threaten the foundations of the project, and the livelihoods of current contributors. **If in doubt, don't do it**, but in particular:

* ⚠️ When implementing an API, rely firstly and primarily on public documentation.
* ⚠️ Do not under any circumstances look at or rely on _leaked_ code, documentation, tools, etc. Material being available somewhere does not mean it is open-source.
* ⚠️ Do not disassemble or decompile components of iPhone OS or other Apple platforms. If you can't figure out how else you would find out how an API should behave, please just don't try to implement it.
* ⚠️ Looking at header files is occasionally necessary, but it should not be your first resort, and you must only use them as a source of simple facts (e.g. what value does a constant have, what type does a type alias resolve to). Do not copy their layout and organization. Do not copy anything you do not need to. Except where the name is part of the ABI or public API, do not copy names.
* ⚠️ Bear in mind that open-source code is still covered by copyright, and so the same caution applies to consulting open-source code. Especially try to avoid looking at the implementation files, unless there is no other option, and do not copy algorithms. (Note however that if it's under a compatible license, we may be able to bring the open-source code into the project _under that license, as a dependency_.)
* ⚠️ If you work or have worked at Apple, or NeXT, or various other organisations, then you may have seen the proprietary source code used in components of iPhone OS. If that's the case, please do not contribute to this project.
* ⚠️ If your employment contract or applicable law in your country means that you don't own the copyright on code you want to contribute to this project, or if for some other reason you may need permission from your employer to contribute to this project: please do obtain that permission before contributing.
