# OpenRouter API Key Setup

## Where to Put Your API Key

### Step 1: Create a `.env` file

Create a `.env` file in the **root directory** of the project (same level as `Cargo.toml`):

```bash
cd /home/vendetta/jamey-3.0
touch .env
```

### Step 2: Add Your API Key

Edit the `.env` file and add your OpenRouter API key:

```bash
# OpenRouter API Configuration
OPENROUTER_API_KEY=sk-or-v1-your-actual-api-key-here

# Optional: Specify the model (defaults to deepseek/deepseek-chat)
OPENROUTER_MODEL=deepseek/deepseek-chat
```

**Important**: Replace `sk-or-v1-your-actual-api-key-here` with your actual API key from OpenRouter.

### Step 3: Get Your API Key

1. Go to [https://openrouter.ai/keys](https://openrouter.ai/keys)
2. Sign in or create an account
3. Create a new API key
4. Copy the key (it starts with `sk-or-v1-`)
5. Paste it into your `.env` file

### Step 4: Verify It's Working

The `.env` file is already in `.gitignore`, so your API key won't be committed to git.

Run the application:

```bash
cargo run
```

If the API key is missing or invalid, you'll see an error message.

## File Structure

```
jamey-3.0/
├── .env                 ← PUT YOUR API KEY HERE
├── .env.example         ← Example file (safe to commit)
├── Cargo.toml
├── src/
└── ...
```

## Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OPENROUTER_API_KEY` | **Yes** | None | Your OpenRouter API key |
| `OPENROUTER_MODEL` | No | `deepseek/deepseek-chat` | The model to use |
| `OPENROUTER_API_URL` | No | `https://openrouter.ai/api/v1` | OpenRouter API endpoint |
| `RUST_LOG` | No | `info` | Logging level |
| `DATABASE_URL` | No | Auto-generated | SQLite database path |

## Security Notes

✅ **DO:**
- Keep your `.env` file local and never commit it
- Use different API keys for development and production
- Rotate your API keys periodically

❌ **DON'T:**
- Commit `.env` to git (it's already in `.gitignore`)
- Share your API key publicly
- Hardcode API keys in source code

## Using the LLM Client

Once configured, you can use the OpenRouter client in your code:

```rust
use jamey_3::{Config, OpenRouterClient};

let config = Config::from_env()?;
let client = OpenRouterClient::new(Arc::new(config));

let response = client.prompt("Hello, how are you?").await?;
```

## Troubleshooting

**Error: "OPENROUTER_API_KEY environment variable is required"**
- Make sure you created a `.env` file in the project root
- Check that the file contains `OPENROUTER_API_KEY=your-key-here`
- Verify there are no extra spaces around the `=` sign

**Error: "OpenRouter API error: 401"**
- Your API key is invalid or expired
- Get a new key from [https://openrouter.ai/keys](https://openrouter.ai/keys)

**Error: "OpenRouter API error: 429"**
- You've hit the rate limit
- Check your OpenRouter account for usage limits

