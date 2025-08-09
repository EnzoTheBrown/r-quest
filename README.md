# Qwest
Qwest is a lightweight command line http client, designed to help you manage and run HTTP-based requests.
Qwest is named after Reqwest the Rust HTTP client.

Qwest is highly configurable through the use of TOML files. To get started, you can run ```qwest create <quest_name>```, which will open your default editor to write the quest specifications.
each quest file is a two part TOML file with. An [api] specification with the name and the description of your quest, and a list of [[requests]] that describe the http request you can run.

```toml
[api]
name = "My API"
base_url = "${BASE_URL}"


[[request]]
name = "doc"
method = "GET"
path = "/docs"

[[request]]
name = "get_users"
method = "GET"
path = "/users"
	[[request.header]]
	key = "Authorization"
	value = "Bearer ${TOKEN}"

[[request]]
name = "login"
method = "POST"
path = "/login"
body = """
{"email": "enzo@tantar.ai", "password": "test1234"}
"""
spell = """
let map = #{ TOKEN: data["access_token"] };
return map;
"""
```
As you can see in the example above, you can add placeholders and those place holder will be replace with the values you provide when running the quest. like so ```bash
qwest run my_app --USER_ID=1234```

You can also specify an env-file:
```bash
qwest run my_app --env-file=.env
```

Qwest also provide you with a way to run scripts after the request is executed, you can use the `script` field in the request to write a script that will be executed after the request is done. The script is written in [Rhai](https://rhai.rs/), a lightweight scripting language.


commands: 

- list: list all existing quests
- create: create a new quest, open your favorite editor to write the quest specifications
- delete: delete an existing quest
- describe: describe a quest
- run: run a quest
