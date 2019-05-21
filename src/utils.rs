use std::str;
use lettre::smtp::SmtpClient;
use lettre::smtp::authentication::{Mechanism, Credentials};
use lettre::Transport;
use lettre_email::EmailBuilder;
use super::SETTINGS;
use super::data_types;
use std::net::SocketAddr;
use jsonwebtoken::{decode, Validation};
use chrono::Utc;
use chrono::prelude::*;
use chrono::offset::TimeZone;

pub fn log(message: &str) {
    let time = chrono::Local::now();
    println!("[{}:{}:{}] {}", time.hour(), time.minute(), time.second(), message);
}

// pub fn get_creds(header: &str) -> data_types::AuthHeader {
//     let bytes = base64::decode(header.trim_start_matches("Basic ")).unwrap_or_default();
//     let decoded: &str = str::from_utf8(&bytes).unwrap_or_default();

//     let creds: Vec<&str> = decoded.split(":").collect();

//     data_types::AuthHeader {
//         email: creds[0].to_owned(),
//         password: creds[1].to_owned(),
//         confirm_password: creds[2].to_owned()
//     }
// }

// pub fn is_auth_header_valid(header: &str) -> bool {
//     let bytes = base64::decode(header.trim_start_matches("Basic ")).unwrap_or_default();
//     let decoded: &str = str::from_utf8(&bytes).unwrap_or_default();

//     let creds: Vec<&str> = decoded.split(":").collect();

//     if creds.len() != 3 {
//         return false;
//     }

//     if !creds[0].contains(".") || !creds[0].contains("@") {
//         return false;
//     }

//     if creds[1] != creds[2] {
//         return false;
//     }

//     true
// }

pub fn send_registration_mail(to: String, username: &str, id: String) {
    let email = EmailBuilder::new()
        .to((to.to_owned(), username))
        .from(SETTINGS.email.noreply.to_string())
        .subject("Theta Radix: Email Verification")
        .html(format!("
            <h2>Theta Radix</h2>
            <h3>Click below to verify your email</h3>
            <br>
            <table width=\"100%\" border=\"0\" cellspacing=\"0\" cellpadding=\"0\">
                <tr>
                    <td>
                        <table border=\"0\" cellspacing=\"0\" cellpadding=\"0\">
                            <tr>
                                <td bgcolor=\"#FF00FF\" style=\"padding: 12px 18px 12px 18px; border-radius:3px\" align=\"center\"><a href=\"http://localhost:8000/activate/{}/{}\" target=\"_blank\" style=\"font-size: 16px; font-family: Helvetica, Arial, sans-serif; font-weight: normal; color: #ffffff; text-decoration: none; display: inline-block;\">Verify &rarr;</a></td>
                            </tr>
                        </table>
                    </td>
                </tr>
            </table>
        ", to, id))
        .build()
        .unwrap();

    let creds = Credentials::new(
        SETTINGS.smtp.username.to_owned(),
        SETTINGS.smtp.password.to_owned(),
    );
    
    // @todo authorize address
    let mut mailer = SmtpClient::new_simple("smtp.mailgun.org")
        .unwrap()
        .credentials(creds)
        .authentication_mechanism(Mechanism::Plain)
        .transport();

    let result = mailer.send(email.into());

    if result.is_ok() {
        println!("Verification email sent to {}", to);
    } else {
        println!("Could not send email to {}: {:?}", to, result);
    }
}


pub fn check_token(token: &str, socket: SocketAddr) -> Result<(), ()> {
    let validation = Validation {
        validate_exp: false,
        ..Validation::default()
    };

    match decode::<data_types::Claims>(&token, SETTINGS.secret.key.as_ref(), &validation) {
        Ok(data) => {
            let date: Vec<&str> = data.claims.exp.split("-").collect();

            if data.claims.ip != socket.ip().to_string() || Utc.ymd(date[0].parse::<i32>().unwrap(), date[1].parse::<u32>().unwrap(), date[2].parse::<u32>().unwrap()) < Utc::now().date() {
                return Err(());
            }

            return Ok(());
        },
        Err(_err) => {
            return Err(());
        }
    }
}