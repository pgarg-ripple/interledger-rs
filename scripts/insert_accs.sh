# configure node 1 with info about 1 and 2

# example.two is at localhost:7070 w/ SE at localhost:3000
# curl http://localhost:7770/accounts -X POST \
#     -d "ilp_address=example.one&asset_code=xyz&asset_scale=9&max_packet_amount=1&settlement_engine_url=http://127.0.0.1:3000&settlement_engine_asset_scale=9&settlement_engine_ilp_address=example.one.se&http_incoming_token=super_secret" \
#     -H "Authorization: Bearer 2173446100652290580079628889802252153830773403671130201871515372"

# example.two is at localhost:7071 w/ SE at localhost:3001
curl http://localhost:7770/accounts -X POST \
    -d "ilp_address=example.two&asset_code=xyz&asset_scale=9&max_packet_amount=1&settlement_engine_url=http://127.0.0.1:3001&settlement_engine_asset_scale=9&settlement_engine_ilp_address=peer.settle.xrpl&http_endpoint=http://127.0.0.1:8770/ilp&http_incoming_token=two&http_outgoing_token=one" \
    -H "Authorization: Bearer 2173446100652290580079628889802252153830773403671130201871515372"

# configure node 2 with info about 1 and 2

# example.one is at localhost:7070 w/ SE at localhost:3000
# curl http://localhost:8770/accounts -X POST \
#     -d "ilp_address=example.two&asset_code=xyz&asset_scale=9&max_packet_amount=1&settlement_engine_url=http://127.0.0.1:3001&settlement_engine_asset_scale=9&settlement_engine_ilp_address=example.two.se" \
#     -H "Authorization: Bearer 1702349080315255078745422539971770693495762245421750086336553265"

curl http://localhost:8770/accounts -X POST \
    -d "ilp_address=example.one&asset_code=xyz&asset_scale=9&max_packet_amount=1&settlement_engine_url=http://127.0.0.1:3000&settlement_engine_asset_scale=9&settlement_engine_ilp_address=peer.settle.xrpl&http_endpoint=http://127.0.0.1:7770/ilp&http_incoming_token=one&http_outgoing_token=two" \
    -H "Authorization: Bearer 1702349080315255078745422539971770693495762245421750086336553265"

# example.two is at localhost:7071 w/ SE at localhost:3001
