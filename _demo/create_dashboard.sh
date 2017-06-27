#!/bin/sh

curl 'http://0.0.0.0:8000/gui-api/dashboard' \
    -H 'content-type: application/json' \
    --data-binary '{"name":"demo","owner_email":"email@subdomain.com","layout":"2x4"}'