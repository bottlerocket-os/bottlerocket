use anyhow::{Context, Result};
use clap::Parser;
use model::test_manager::TestManager;
use model::SecretName;

/// Add a testsys object to the testsys cluster.
#[derive(Debug, Parser)]
pub(crate) struct Add {
    #[clap(subcommand)]
    command: AddCommand,
}

#[derive(Debug, Parser)]
enum AddCommand {
    /// Add a secret to the testsys cluster.
    Secret(AddSecret),
}

impl Add {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
        match self.command {
            AddCommand::Secret(add_secret) => add_secret.run(client).await,
        }
    }
}

/// Add a secret to the cluster.
#[derive(Debug, Parser)]
pub(crate) struct AddSecret {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    /// Create a secret for image pulls.
    Image(AddSecretImage),
    /// Create a secret from key value pairs.
    Map(AddSecretMap),
}

impl AddSecret {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
        match self.command {
            Command::Image(add_secret_image) => add_secret_image.run(client).await,
            Command::Map(add_secret_map) => add_secret_map.run(client).await,
        }
    }
}

/// Add a `Secret` with key value pairs.
#[derive(Debug, Parser)]
pub(crate) struct AddSecretMap {
    /// Name of the secret
    #[clap(short, long)]
    name: SecretName,

    /// Key value pairs for secrets. (Key=value)
    #[clap(parse(try_from_str = parse_key_val))]
    args: Vec<(String, String)>,
}

impl AddSecretMap {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
        client
            .create_secret(&self.name, self.args)
            .await
            .context("Unable to create secret")?;
        println!("Successfully added '{}' to secrets.", self.name);
        Ok(())
    }
}

fn parse_key_val(s: &str) -> Result<(String, String)> {
    let mut iter = s.splitn(2, '=');
    let key = iter.next().context("Key is missing")?;
    let value = iter.next().context("Value is missing")?;
    Ok((key.to_string(), value.to_string()))
}

/// Add a secret to the testsys cluster for image pulls.
#[derive(Debug, Parser)]
pub(crate) struct AddSecretImage {
    /// Controller image pull username
    #[clap(long, short = 'u')]
    pull_username: String,

    /// Controller image pull password
    #[clap(long, short = 'p')]
    pull_password: String,

    /// Image uri
    #[clap(long = "image-uri", short)]
    image_uri: String,

    /// Controller image uri
    #[clap(long, short = 'n')]
    secret_name: String,
}

impl AddSecretImage {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
        client
            .create_image_pull_secret(
                &self.secret_name,
                &self.pull_username,
                &self.pull_password,
                &self.image_uri,
            )
            .await
            .context("Unable to create pull secret")?;

        println!("The secret was added.");

        Ok(())
    }
}
