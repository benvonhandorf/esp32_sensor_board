use esp_idf_svc::{mqtt::client::*, sys::EspError};

const MQTT_URL: str = "mqtt://littlerascal.local:1883";
const MQTT_USER: str = "status_light";
const MQTT_PASSWORD: str = "";
const MQTT_CLIENT_ID: str = "esp32-123";

pub fn mqtt_create() -> Result<(EspMqttClient<'static>, EspMqttConnection), EspError> {
    

    let (mqtt_client, mqtt_conn) = EspMqttClient::new(
        &MQTT_URL,
        &MqttClientConfiguration {
            username: Some(&MQTT_USERNAME),
            password: Some(&MQTT_PASSWORD),
            client_id: Some(&MQTT_CLIENT_ID),
            ..Default::default()
        },
    )?;

    Ok((mqtt_client, mqtt_conn))
}

pub(crate) fn mqtt_post(mqtt_tuple: &(EspMqttClient<'_>, EspMqttConnection), topic: &str, value: &str) -> Result<(), EspError> {
    let (mqtt_client, mqtt_connection) = mqtt_tuple;

    
}

