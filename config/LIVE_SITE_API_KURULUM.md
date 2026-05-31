# Canlı API Connector (KN-14)

`system_connectors.json` içindeki `live_site_api` varsayılan olarak kapalıdır.

Etkinleştirmek için:

1. `base_url` alanını gerçek API kök adresi ile doldurun.
2. Ortam değişkeni: `LIVE_SITE_API_KEY`
3. `enabled: true` yapın (yalnızca yukarıdakiler tamamlandıktan sonra).

Panel health-check: TCP ping + key varlığı.
