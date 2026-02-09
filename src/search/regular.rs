use anyhow::Result;
use urlencoding::encode;

use crate::{open_tab, focus_brave};
use super::prompt;

pub async fn run() -> Result<()> {
    let query = prompt("Search Brave");
    if query.is_empty() {
        return Ok(());
    }

    let url = format!(
        "https://search.brave.com/search?q={}",
        encode(&query)
    );

    open_tab(&url).await?;
    focus_brave();

    Ok(())
}
