use reqwest::Client;
use std::{
    fs::File,
    io::{self, BufReader, Write},
    process::ExitCode,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AuthData {
    email: String,
    password: String,
    token: Option<String>,
}

// Read authentication data from JSON file
fn read_auth_data_from_file() -> io::Result<AuthData> {
    let file = File::open("auth.json")?;
    let reader = BufReader::new(file);
    let auth_data: AuthData = serde_json::from_reader(reader)?;
    Ok(auth_data)
}

async fn make_private_request(url: &str, token: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let response = client
        .get(url.to_string())
        .header("Authorization", token.to_string())
        .header("User-Agent", "insomnia/8.4.0")
        .send()
        .await?;

    // Check if the response is successful
    if response.status().is_success() {
        match url {
            "http://localhost:8000/api/v1/private/profile/my" => {
                let text = response.text().await?;
                let profile: serde_json::Value = serde_json::from_str(&text).unwrap();
                let first_name = profile["first_name"].as_str().unwrap_or("Unknown");
                let id = profile["id"].as_str().unwrap_or("Unknown");
                println!("Hi, {}!", first_name);
                println!("Your id: {}", id);
            }
            "http://localhost:8000/api/v1/private/profile/my/notifications" => {
                println!("Notifications:");
                let notifications: serde_json::Value =
                    serde_json::from_str(&response.text().await?).unwrap();
                for notification in notifications.as_array().unwrap_or(&vec![]) {
                    let message = notification["message"].as_str().unwrap_or("No message");
                    let author = notification["created_by"].as_str().unwrap_or("Unknown");
                    let time = notification["created_at"].as_str().unwrap_or("Unknown");
                    println!("--------------");
                    println!("Message: {}", message);
                    println!("Author: {}", author);
                    println!("Time: {}", time);
                    println!("____________");
                }
            }
            _ => {
                let text = response.text().await?;
                let formatted_json = serde_json::to_string_pretty(&text).unwrap();
                println!("{}", formatted_json);
            }
        }
    } else {
        println!("Error: {}", response.status());
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Read authentication data from file
    let auth_data = match read_auth_data_from_file() {
        Ok(data) => data,
        Err(_) => {
            println!("Error: Authentication data not found.");
            return;
        }
    };

    // Prompt the user for their choice
    loop {
        println!("");
        println!("Choose an action:");
        println!("1. Get my profile");
        println!("2. Get my documents");
        println!("3. Get notifications");
        println!("4. Get connected devices");
        println!("5. Exit");

        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read choice");
        let choice: u32 = match choice.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid choice");
                return;
            }
        };

        // Construct URLs for the requests based on user choice
        let (url, action) = match choice {
            1 => ("http://localhost:8000/api/v1/private/profile/my", "profile"),
            2 => (
                "http://localhost:8000/api/v1/private/profile/my/documents",
                "documents",
            ),
            3 => (
                "http://localhost:8000/api/v1/private/profile/my/notifications",
                "notifications",
            ),
            4 => (
                "http://localhost:8000/api/v1/private/profile/my/devices",
                "connected devices",
            ),
            5 => {
                println!("Exiting...");
                return;
            }
            _ => {
                println!("Invalid choice");
                return;
            }
        };
        println!("");

        // Make requests using the token from authentication data
        if let Some(token) = auth_data.token.clone() {
            if let Err(err) = make_private_request(url, &token).await {
                println!("Error making {} request: {:?}", action, err);
            }
        } else {
            println!("Error: No authentication token found.");
        }
    }
}
