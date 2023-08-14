#include <stdio.h>
#include "esp_log.h"

void app_main(void)
{
    ESP_LOGW("MAIN", "Hello world!\n");

    while(1) {

        ESP_LOGW("MAIN", "Hello world!\n");
    }

}