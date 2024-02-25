Clutha
===

Clutha is a simple replacement for the lately departed Clyde of Discord.

Set up
---

  1. Login in to the Discord web site and create a new application.  Configure it as appropriate (private application, bot, etc.).  Generate a Token.

  2. Log in to Google Gemini and create an account, generating an API Key.

  3. Run `cargo build` to compile Clutha.

  4. Set the following environment variables:

```
    export DISCORD_TOKEN=<token from step 1>
    export API_KEY=<api key from step 2>
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

  * `~version` Display the version information
  * `-shutdown` Shut down the Clutha application, signing off from all Discord servers
  * `~ping` Very that the application is running and responsive.

Caveats and disclaimers
---

Not designed for use by general public.
Use at own risk.
