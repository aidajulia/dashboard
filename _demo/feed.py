#!/usr/bin/env python3
import json
import logging
import os

logging.basicConfig(level=logging.DEBUG)

import requests

PUSH_URL = "http://{}/api/dashboard/demo/tile".format(
    os.environ.get('DASHBOARD_IP_PORT', 'localhost:8000')
)
PUSH_URL += "/{tile_id}"
DEMO_ADDRESS = "85.255.1.138:80"

tiles_data = {}

json_data = """{
    "labels":[
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July"
    ],
    "datasets":[
        {
            "label":"My First dataset",
            "fill":false,
            "lineTension":0.1,
            "backgroundColor":"rgba(75,192,192,0.4)",
            "borderColor":"rgba(75,192,192,1)",
            "borderCapStyle":"butt",
            "borderDash":[

            ],
            "borderDashOffset":0,
            "borderJoinStyle":"miter",
            "pointBorderColor":"rgba(75,192,192,1)",
            "pointBackgroundColor":"#fff",
            "pointBorderWidth":1,
            "pointHoverRadius":5,
            "pointHoverBackgroundColor":"rgba(75,192,192,1)",
            "pointHoverBorderColor":"rgba(220,220,220,1)",
            "pointHoverBorderWidth":2,
            "pointRadius":1,
            "pointHitRadius":10,
            "data":[
                65,
                59,
                80,
                81,
                56,
                55,
                40
            ],
            "spanGaps":false
        }
    ]
}"""
tiles_data['tile-chart1'] = tiles_data['tile-chart-line'] = {
    'tile-data': {
        "header": "Line chart!!",
        "type": "line",
        "options": {},
        "data": json.loads(json_data),
    }
}

json_data = """{
    "labels":[
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July"
    ],
    "datasets":[
        {
            "label":"My First dataset",
            "backgroundColor":[
                "rgba(255, 99, 132, 0.2)",
                "rgba(54, 162, 235, 0.2)",
                "rgba(255, 206, 86, 0.2)",
                "rgba(75, 192, 192, 0.2)",
                "rgba(153, 102, 255, 0.2)",
                "rgba(255, 159, 64, 0.2)"
            ],
            "borderColor":[
                "rgba(255,99,132,1)",
                "rgba(54, 162, 235, 1)",
                "rgba(255, 206, 86, 1)",
                "rgba(75, 192, 192, 1)",
                "rgba(153, 102, 255, 1)",
                "rgba(255, 159, 64, 1)"
            ],
            "borderWidth":1,
            "data":[
                65,
                59,
                80,
                81,
                56,
                55,
                40
            ]
        }
    ]
}"""
tiles_data['tile-chart2'] = tiles_data['tile-chart-bar'] = {
    'tile-data': {
        "header": "Bar chart!!",
        "type": "bar",
        "options": {},
        "data": json.loads(json_data),
    }
}

json_data = """{
    "labels":[
        "Eating",
        "Drinking",
        "Sleeping",
        "Designing",
        "Coding",
        "Cycling",
        "Running"
    ],
    "datasets":[
        {
            "label":"My First dataset",
            "backgroundColor":"rgba(179,181,198,0.2)",
            "borderColor":"rgba(179,181,198,1)",
            "pointBackgroundColor":"rgba(179,181,198,1)",
            "pointBorderColor":"#fff",
            "pointHoverBackgroundColor":"#fff",
            "pointHoverBorderColor":"rgba(179,181,198,1)",
            "data":[
                65,
                59,
                90,
                81,
                56,
                55,
                40
            ]
        },
        {
            "label":"My Second dataset",
            "backgroundColor":"rgba(255,99,132,0.2)",
            "borderColor":"rgba(255,99,132,1)",
            "pointBackgroundColor":"rgba(255,99,132,1)",
            "pointBorderColor":"#fff",
            "pointHoverBackgroundColor":"#fff",
            "pointHoverBorderColor":"rgba(255,99,132,1)",
            "data":[
                28,
                48,
                40,
                19,
                96,
                27,
                100
            ]
        }
    ]
}"""
tiles_data["tile-chart-radar"] = {
    "tile-data": {
        "header": "Radar tile!!",
        "type": "radar",
        "options": {},
        "data": json.loads(json_data),
    }
}


json_data = """{
    "datasets":[
        {
            "data":[
                11,
                16,
                7,
                3,
                14
            ],
            "backgroundColor":[
                "#FF6384",
                "#4BC0C0",
                "#FFCE56",
                "#E7E9ED",
                "#36A2EB"
            ],
            "label":"My dataset"
        }
    ],
    "labels":[
        "Red",
        "Green",
        "Yellow",
        "Grey",
        "Blue"
    ]
}"""
tiles_data['tile-chart-polar'] = {
    'tile-data': {
        "header": "Polar chart!!",
        "type": "polarArea",
        "options": {},
        "data": json.loads(json_data),
    }
}

json_data = """{
    "labels":[
        "Red",
        "Blue",
        "Yellow"
    ],
    "datasets":[
        {
            "data":[
                300,
                50,
                100
            ],
            "backgroundColor":[
                "#FF6384",
                "#36A2EB",
                "#FFCE56"
            ],
            "hoverBackgroundColor":[
                "#FF6384",
                "#36A2EB",
                "#FFCE56"
            ]
        }
    ]
}"""
tiles_data['tile-chart3'] = tiles_data['tile-chart-pie'] = {
    'tile-data': {
        "header": "Pie chart!!",
        "type": "pie",
        "options": {},
        "data": json.loads(json_data),
    }
}

tiles_data['tile-chart-doughnut'] = {
    'tile-data': {
        "header": "Doughnut chart!!",
        "type": "doughnut",
        "options": {},
        "data": json.loads(json_data),
    }
}

json_data = """{
    "datasets":[
        {
            "label":"First Dataset",
            "data":[
                {
                    "x":20,
                    "y":30,
                    "r":15
                },
                {
                    "x":40,
                    "y":10,
                    "r":10
                }
            ],
            "backgroundColor":"#FF6384",
            "hoverBackgroundColor":"#FF6384"
        }
    ]
}"""
tiles_data['tile-chart-bubble'] = {
    'tile-data': {
        "header": "Bubble chart!!",
        "type": "bubble",
        "options": {},
        "data": json.loads(json_data),
    }
}


tiles_data["tile-image1"] = {
    "tile-data": {
        "header": "Rust logo",
        "header": "rust",
        "imageSrc": "https://www.rust-lang.org/logos/rust-logo-256x256.png",
    },
}
tiles_data['tile-image2'] = {
    'tile-data': {
        "header": "Polymer logo",
        "imageSrc": "https://www.polymer-project.org/images/logos/p-logo.png",
    }
}
tiles_data['tile-image3'] = {
    'tile-data': {
        "header": "vf",
        "imageSrc": "https://media.giphy.com/media/D8UYWEpnUpJ4Y/giphy.gif",
    }
}


tiles_data['tile-markdown1'] = tiles_data['tile-markdown-simple'] = {
    'tile-data': {
        "markdown": "`Markdown` is totally _awesome_!"
    }
}

tiles_data['tile-markdown2'] = tiles_data['tile-markdown-listing'] = {
    'tile-data': {
        "markdown": """# Header 1
## Header 2
### Header 3
#### Header 4 ####
##### Header 5 #####

1. Item
2. Item
   * Mixed
   * Mixed
3. Item
"""
    }
}
tiles_data['tile-markdown3'] = {
    'tile-data': {
        "markdown": """# Tile docs
Each of links below contains demo (to see the demo click button in top-right corner after link clicked).
* [Tile Chart](http://{demo_address}/components/dashboard-toolkit/#tile-chart)
* [Tile Image](http://{demo_address}/components/dashboard-toolkit/#tile-image)
* [Tile Markdown](http://{demo_address}/components/dashboard-toolkit/#tile-markdown)
* [Tile Value](http://{demo_address}/components/dashboard-toolkit/#tile-value)
* [Other Dashboards](http://{demo_address}/components/dashboard-toolkit/#dashboard-ws)
""".format(demo_address=DEMO_ADDRESS)
    }
}
print(tiles_data['tile-markdown3'])


tiles_data['tile-value1'] = {
    'tile-data': {
        "value": "100%"
    }
}

tiles_data['tile-value2'] = {
    'tile-data': {
        "value": "50%",
        "color": "yellow",
        "backgroundColor": "#12B0C5",
    }
}


client = requests.Session()
client.headers.update(
    {'Authorization': os.environ.get('DASHBOARD_DASHBOARD_TOKEN', 'change-me')}
)

for tile_id, data in sorted(tiles_data.items()):
    jsoned_data = json.dumps(data)
    url = PUSH_URL.format(tile_id=tile_id)
    response = client.post(url, data=jsoned_data)
    print(tile_id, response.status_code)
    if response.status_code != 201:
        print(response.content)
