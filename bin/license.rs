// create_license.rs (это отдельный скрипт, не часть вашего EGUI приложения)
use base64::{Engine as _, engine::general_purpose};
use chrono::{Local, NaiveDate};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};
use std::fs;

// Структура данных лицензии
#[derive(Serialize, Deserialize, Debug)]
struct LicenseData {
    licensed_to: String,
    expiration_date: NaiveDate,
    // Можно добавить другие поля, например, версию программы, ID пользователя и т.д.
}

// Структура для хранения лицензионного файла (данные + подпись)
#[derive(Serialize, Deserialize, Debug)]
struct SignedLicense {
    data: String,      // Base64-кодированные данные лицензии
    signature: String, // Base64-кодированная подпись
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Загружаем закрытый ключ
    let private_key_bytes = fs::read("private_key.bin")?;
    let keypair = Keypair::from_bytes(&private_key_bytes)?;

    // 2. Создаем данные лицензии
    let licensed_to = "Имя Пользователя".to_string(); // Замените на реальное имя
    let expiration_date = NaiveDate::from_ymd_opt(2027, 4, 15).unwrap(); // Дата истечения срока

    let license_data = LicenseData {
        licensed_to,
        expiration_date,
    };

    // 3. Сериализуем данные лицензии в JSON
    let serialized_data = serde_json::to_string(&license_data)?;
    let data_bytes = serialized_data.as_bytes();

    // 4. Подписываем данные
    let signature: Signature = keypair.sign(data_bytes);

    // 5. Кодируем данные и подпись в Base64 для сохранения в файл
    let encoded_data = general_purpose::STANDARD.encode(data_bytes);
    let encoded_signature = general_purpose::STANDARD.encode(signature.to_bytes());

    let signed_license = SignedLicense {
        data: encoded_data,
        signature: encoded_signature,
    };

    // 6. Сохраняем подписанную лицензию в файл (например, в JSON формате)
    let license_file_content = serde_json::to_string_pretty(&signed_license)?;
    fs::write("license.json", license_file_content)?;

    println!("Лицензионный файл 'license.json' создан успешно.");
    println!("Содержимое лицензии: {:?}", license_data);

    Ok(())
}
