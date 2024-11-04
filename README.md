# API version 1

_The first iteration of an API interface, version 1, written in Rust_

- - -

The API takes a POST request with a JSON body.

The following fields can be set when searching:

| Name             | Type                            | Default | Example              | Description                                                                                                                                                                                                                                                            |
|------------------|---------------------------------|---------|----------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| search\_type     | `"score"` \| _percentage value_ | `100%`  | `score`              | search\_type can either be `score`, referring to scoring based on best hit a percentage value from 0% - 100% referring to Elasticsarch's [`minimum_should_match`](https://www.elastic.co/guide/en/elasticsearch/reference/current/query-dsl-minimum-should-match.html) |
| search\_field    | String                          | `None`  | `title`              | If set, will match the query search on the set field. The available fields are mentioned below.                                                                                                                                                                        |
| query            | String                          | `None`  | `Debian Linux ISO`   | This is the search query itself                                                                                                                                                                                                                                        |
| order\_by        | String                          | `None`  | `seeders`            | The field to order by. Normally most people order by seeders or peers in descending order.                                                                                                                                                                             |
| order\_direction | `"desc"` \| `"asc"`             | `desc`  | `asc`                | This is the direction you want to order by in. Descending, biggest number first. Ascending, Smallest number first.                                                                                                                                                     |
| categories       | Integer\[\]                     | `None`  | `[5001000, 5004000]` | The category ids you want to search on. There are so many ids, better have a look at the [RSS page](https://knaben.eu/rss), clicking on _Toggle show categories_ to find the IDs                                                                                       |
| from             | Integer                         | `0`     | `0`                  | This is the scroll offset. If you have a size of 10, and want to scroll to the next page, you can do from: 10, size: 10. Then from: 20, size 10 etc...                                                                                                                 |
| size             | Integer                         | `150`   | `50`                 | The amount of results to return. Maximum size is 300                                                                                                                                                                                                                   |
| hide\_unsafe     | Boolean                         | `true`  | `false`              | Filters out very old results, and results that has a relative high potential virus score.                                                                                                                                                                              |
| hide\_xxx        | Boolean                         | `false` | `true`               | To hide porn or not to hide porn.                                                                                                                                                                                                                                      |

  ---

**Example request**
```
POST /v1
{
  "search_type": "100%",
  "search_field": "title",
  "query": "dubioza kolektiv",
  "order_by": "peers",
  "order_direction": "desc",
  "categories": [
    1004000, 1001000
  ],
  "from": 0,
  "size": 150,
  "hide_unsafe": true,
  "hide_xxx": true
}
```  

**Example response**
```
{
  "max_score": null,
  "total": {
    "relation": "eq",
    "value": 7
  },
  "hits": [
    {
      "bytes": 8734638,
      "cachedOrigin": "The Pirate Bay (proxy)",
      "category": "Audio / MP3",
      "categoryId": [
        1000000,
        1001000
      ],
      "date": "2015-11-08T00:00:00",
      "details": "https://knaben.xyz/thepiratebay/description.php?id=12724322",
      "hash": "B06B8470FF02B031433C94D5434C6795B5FA092E",
      "id": "918da9da460250d3d1bd166e8ddb4704f3a6adb3",
      "lastSeen": "2024-11-04T17:55:15",
      "magnetUrl": "magnet:?xt=urn:btih:B06B8470FF02B031433C94D5434C6795B5FA092E&dn=Dubioza+Kolektiv+-+Free+MP3.mp3&tr=http%3A%2F%2Fp4p.arenabg.com%3A1337%2Fannounce&tr=udp%3A%2F%2F47.ip-51-68-199.eu%3A6969%2Fannounce&tr=udp%3A%2F%2F9.rarbg.me%3A2780%2Fannounce&tr=udp%3A%2F%2F9.rarbg.to%3A2710%2Fannounce&tr=udp%3A%2F%2F9.rarbg.to%3A2730%2Fannounce&tr=udp%3A%2F%2F9.rarbg.to%3A2920%2Fannounce&tr=udp%3A%2F%2Fopen.stealth.si%3A80%2Fannounce&tr=udp%3A%2F%2Fopentracker.i2p.rocks%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.coppersurfer.tk%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.cyberia.is%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.dler.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.internetwarriors.net%3A1337%2Fannounce&tr=udp%3A%2F%2Ftracker.leechers-paradise.org%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.openbittorrent.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Ftracker.pirateparty.gr%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.tiny-vps.com%3A6969%2Fannounce&tr=udp%3A%2F%2Ftracker.torrent.eu.org%3A451%2Fannounce",
      "peers": 1,
      "score": null,
      "seeders": 0,
      "title": "Dubioza Kolektiv - Free MP3.mp3",
      "tracker": "The Pirate Bay",
      "trackerId": "thepiratebay",
      "virusDetection": 0.16737702077833227
    },
    {
      "bytes": 76902560,
      "cachedOrigin": "1337x",
      "category": "Audio / MP3",
      "categoryId": [
        1001000,
        1000000
      ],
      "date": "2018-08-11T00:00:00+02:00",
      "details": "https://1337x.to/torrent/3168985/Dubioza-Kolektiv-Pjesmice-za-djecu-i-odrasle-MP3-320kbps-2017-MRSK/",
      "hash": null,
      "id": "edab57e04c7a0d9c62521bc4fe17e16d2daa11cc",
      "lastSeen": "2024-11-04T18:34:44",
      "link": "https://knaben.eu/live/dl/1337x/?path=Q2ZESjhDWHZSYU9Hcm9sQWdfMURDakxrWldKcDZKdmo4UjNmTzQtUFdWZUdibnB5N0lnckVoY0RZS0o3N2Y3ZFR1NHdrQlduVmhWazBEY0pFUl9jYjd5QzBoZUVjR1dJVGw5ZGN3RlBQS011dkVNZ19KaVU2My1BY0RyRjItdTRCSHNzTVE5QjROVE5XMi1PWXFJR0piQXg4UDI3U3k0NVpReEdKazhiQi1qTlo2UmVHeE1tcElFUlNjN3B5MTJxMmdnbXVzWWxtUE1NX3lPc25xSldZZkIxTm5oVXhrbmdrdXVZMERmRnN6d2ZYaHM5LVl5anFVaXRoVVk5VDZaNjhHWTRTQQ&file=Dubioza+Kolektiv+++Pjesmice+za+djecu+i+odrasle+%5BMP3+320kbps%5D+(2017)+%5BMRSK%5D&knabensource=https%3A%2F%2F1337x.to%2Ftorrent%2F3168985%2FDubioza-Kolektiv-Pjesmice-za-djecu-i-odrasle-MP3-320kbps-2017-MRSK%2F",
      "magnetUrl": null,
      "peers": 0,
      "score": null,
      "seeders": 4,
      "title": "Dubioza Kolektiv   Pjesmice za djecu i odrasle [MP3 320kbps] (2017) [MRSK]",
      "tracker": "1337x",
      "trackerId": "1337x",
      "virusDetection": 0.28133041133809267
    },
    {
      ...
    }
  ]
}
```
