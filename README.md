Clutha
===

[![Rust build](https://github.com/ejrh/clutha/actions/workflows/rust-build.yml/badge.svg)](https://github.com/ejrh/clutha/actions/workflows/rust-build.yml)
[![Rust tests](https://github.com/ejrh/clutha/actions/workflows/rust-tests.yml/badge.svg)](https://github.com/ejrh/clutha/actions/workflows/rust-tests.yml)
[![Rust Clippy](https://github.com/ejrh/clutha/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/ejrh/clutha/actions/workflows/rust-clippy.yml)


Clutha is a simple replacement for the lately departed Clyde of Discord.

Set up
---

  1. Login in to the Discord web site and create a new application.  Configure it as appropriate (private application, bot, etc.).  Generate a Token.

  2. Log in to Google Gemini and create an account, generating an API Key.

  3. Run `cargo build` to compile Clutha.

  4. Set the following environment variables:

```
    export DISCORD_TOKEN=<token from step 1>
    export GEMINI_API_KEY=<api key from step 2>
```

  5. Run Clutha by typing `cargo run`.

Functionality
---

Currently very little to none.

The main intention of Clutha is to provide AI chat for the purposes of:

1. Providing general knowledge information.

2. Facilitating text based games.

3. Entertaining lonely people who can't find any humans to talk to.

Clutha currently uses Google Gemini for AI functionality.

Commands
--- 

There are a number of commands that control Clutha.  They are prefixed by `~` and are not
considered part of the AI conversation.

A list of commands available to a user can be displayed with `~help`.  Commands in the Admin
group require ownership of the bot (i.e. being the Discord user that owns the Discord App that
Clutha is logged in as).

Caveats and disclaimers
---

Not designed for use by general public.
Install or run at own risk.

Do not operate heavy machinery after reading this disclaimer.
