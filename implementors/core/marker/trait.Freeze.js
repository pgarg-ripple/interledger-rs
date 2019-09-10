(function() {var implementors = {};
implementors["interledger_api"] = [{text:"impl Freeze for <a class=\"struct\" href=\"interledger_api/struct.AccountSettings.html\" title=\"struct interledger_api::AccountSettings\">AccountSettings</a>",synthetic:true,types:["interledger_api::AccountSettings"]},{text:"impl !Freeze for <a class=\"struct\" href=\"interledger_api/struct.AccountDetails.html\" title=\"struct interledger_api::AccountDetails\">AccountDetails</a>",synthetic:true,types:["interledger_api::AccountDetails"]},{text:"impl&lt;S, I&gt; !Freeze for <a class=\"struct\" href=\"interledger_api/struct.NodeApi.html\" title=\"struct interledger_api::NodeApi\">NodeApi</a>&lt;S, I&gt;",synthetic:true,types:["interledger_api::NodeApi"]},];
implementors["interledger_btp"] = [{text:"impl&lt;O, A&gt; !Freeze for <a class=\"struct\" href=\"interledger_btp/struct.BtpOutgoingService.html\" title=\"struct interledger_btp::BtpOutgoingService\">BtpOutgoingService</a>&lt;O, A&gt;",synthetic:true,types:["interledger_btp::service::BtpOutgoingService"]},{text:"impl&lt;I, O, A&gt; !Freeze for <a class=\"struct\" href=\"interledger_btp/struct.BtpService.html\" title=\"struct interledger_btp::BtpService\">BtpService</a>&lt;I, O, A&gt;",synthetic:true,types:["interledger_btp::service::BtpService"]},{text:"impl&lt;'a&gt; Freeze for <a class=\"struct\" href=\"interledger_btp/struct.BtpOpenSignupAccount.html\" title=\"struct interledger_btp::BtpOpenSignupAccount\">BtpOpenSignupAccount</a>&lt;'a&gt;",synthetic:true,types:["interledger_btp::BtpOpenSignupAccount"]},];
implementors["interledger_http"] = [{text:"impl&lt;S, O, A&gt; !Freeze for <a class=\"struct\" href=\"interledger_http/struct.HttpClientService.html\" title=\"struct interledger_http::HttpClientService\">HttpClientService</a>&lt;S, O, A&gt;",synthetic:true,types:["interledger_http::client::HttpClientService"]},{text:"impl&lt;S, T&gt; Freeze for <a class=\"struct\" href=\"interledger_http/struct.HttpServerService.html\" title=\"struct interledger_http::HttpServerService\">HttpServerService</a>&lt;S, T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Freeze,&nbsp;</span>",synthetic:true,types:["interledger_http::server::HttpServerService"]},];
implementors["interledger_ildcp"] = [{text:"impl Freeze for <a class=\"struct\" href=\"interledger_ildcp/struct.IldcpRequest.html\" title=\"struct interledger_ildcp::IldcpRequest\">IldcpRequest</a>",synthetic:true,types:["interledger_ildcp::packet::IldcpRequest"]},{text:"impl !Freeze for <a class=\"struct\" href=\"interledger_ildcp/struct.IldcpResponse.html\" title=\"struct interledger_ildcp::IldcpResponse\">IldcpResponse</a>",synthetic:true,types:["interledger_ildcp::packet::IldcpResponse"]},{text:"impl&lt;'a&gt; Freeze for <a class=\"struct\" href=\"interledger_ildcp/struct.IldcpResponseBuilder.html\" title=\"struct interledger_ildcp::IldcpResponseBuilder\">IldcpResponseBuilder</a>&lt;'a&gt;",synthetic:true,types:["interledger_ildcp::packet::IldcpResponseBuilder"]},{text:"impl&lt;I, A&gt; Freeze for <a class=\"struct\" href=\"interledger_ildcp/struct.IldcpService.html\" title=\"struct interledger_ildcp::IldcpService\">IldcpService</a>&lt;I, A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;I: Freeze,&nbsp;</span>",synthetic:true,types:["interledger_ildcp::server::IldcpService"]},];
implementors["interledger_packet"] = [{text:"impl !Freeze for <a class=\"struct\" href=\"interledger_packet/struct.Address.html\" title=\"struct interledger_packet::Address\">Address</a>",synthetic:true,types:["interledger_packet::address::Address"]},{text:"impl Freeze for <a class=\"struct\" href=\"interledger_packet/struct.ErrorCode.html\" title=\"struct interledger_packet::ErrorCode\">ErrorCode</a>",synthetic:true,types:["interledger_packet::error::ErrorCode"]},{text:"impl Freeze for <a class=\"struct\" href=\"interledger_packet/struct.MaxPacketAmountDetails.html\" title=\"struct interledger_packet::MaxPacketAmountDetails\">MaxPacketAmountDetails</a>",synthetic:true,types:["interledger_packet::packet::MaxPacketAmountDetails"]},{text:"impl !Freeze for <a class=\"struct\" href=\"interledger_packet/struct.Fulfill.html\" title=\"struct interledger_packet::Fulfill\">Fulfill</a>",synthetic:true,types:["interledger_packet::packet::Fulfill"]},{text:"impl !Freeze for <a class=\"struct\" href=\"interledger_packet/struct.Prepare.html\" title=\"struct interledger_packet::Prepare\">Prepare</a>",synthetic:true,types:["interledger_packet::packet::Prepare"]},{text:"impl !Freeze for <a class=\"struct\" href=\"interledger_packet/struct.Reject.html\" title=\"struct interledger_packet::Reject\">Reject</a>",synthetic:true,types:["interledger_packet::packet::Reject"]},{text:"impl&lt;'a&gt; Freeze for <a class=\"struct\" href=\"interledger_packet/struct.FulfillBuilder.html\" title=\"struct interledger_packet::FulfillBuilder\">FulfillBuilder</a>&lt;'a&gt;",synthetic:true,types:["interledger_packet::packet::FulfillBuilder"]},{text:"impl&lt;'a&gt; !Freeze for <a class=\"struct\" href=\"interledger_packet/struct.PrepareBuilder.html\" title=\"struct interledger_packet::PrepareBuilder\">PrepareBuilder</a>&lt;'a&gt;",synthetic:true,types:["interledger_packet::packet::PrepareBuilder"]},{text:"impl&lt;'a&gt; Freeze for <a class=\"struct\" href=\"interledger_packet/struct.RejectBuilder.html\" title=\"struct interledger_packet::RejectBuilder\">RejectBuilder</a>&lt;'a&gt;",synthetic:true,types:["interledger_packet::packet::RejectBuilder"]},{text:"impl Freeze for <a class=\"enum\" href=\"interledger_packet/enum.AddressError.html\" title=\"enum interledger_packet::AddressError\">AddressError</a>",synthetic:true,types:["interledger_packet::address::AddressError"]},{text:"impl Freeze for <a class=\"enum\" href=\"interledger_packet/enum.ErrorClass.html\" title=\"enum interledger_packet::ErrorClass\">ErrorClass</a>",synthetic:true,types:["interledger_packet::error::ErrorClass"]},{text:"impl Freeze for <a class=\"enum\" href=\"interledger_packet/enum.ParseError.html\" title=\"enum interledger_packet::ParseError\">ParseError</a>",synthetic:true,types:["interledger_packet::errors::ParseError"]},{text:"impl !Freeze for <a class=\"enum\" href=\"interledger_packet/enum.Packet.html\" title=\"enum interledger_packet::Packet\">Packet</a>",synthetic:true,types:["interledger_packet::packet::Packet"]},{text:"impl Freeze for <a class=\"enum\" href=\"interledger_packet/enum.PacketType.html\" title=\"enum interledger_packet::PacketType\">PacketType</a>",synthetic:true,types:["interledger_packet::packet::PacketType"]},];
implementors["interledger_router"] = [{text:"impl&lt;S, O&gt; !Freeze for <a class=\"struct\" href=\"interledger_router/struct.Router.html\" title=\"struct interledger_router::Router\">Router</a>&lt;S, O&gt;",synthetic:true,types:["interledger_router::router::Router"]},];
implementors["interledger_service"] = [{text:"impl Freeze for <a class=\"struct\" href=\"interledger_service/struct.AuthToken.html\" title=\"struct interledger_service::AuthToken\">Auth</a>",synthetic:true,types:["interledger_service::auth::token::Auth"]},{text:"impl Freeze for <a class=\"struct\" href=\"interledger_service/struct.Username.html\" title=\"struct interledger_service::Username\">Username</a>",synthetic:true,types:["interledger_service::auth::username::Username"]},{text:"impl&lt;A&gt; !Freeze for <a class=\"struct\" href=\"interledger_service/struct.IncomingRequest.html\" title=\"struct interledger_service::IncomingRequest\">IncomingRequest</a>&lt;A&gt;",synthetic:true,types:["interledger_service::IncomingRequest"]},{text:"impl&lt;A&gt; !Freeze for <a class=\"struct\" href=\"interledger_service/struct.OutgoingRequest.html\" title=\"struct interledger_service::OutgoingRequest\">OutgoingRequest</a>&lt;A&gt;",synthetic:true,types:["interledger_service::OutgoingRequest"]},{text:"impl&lt;F, A&gt; Freeze for <a class=\"struct\" href=\"interledger_service/struct.ServiceFn.html\" title=\"struct interledger_service::ServiceFn\">ServiceFn</a>&lt;F, A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: Freeze,&nbsp;</span>",synthetic:true,types:["interledger_service::ServiceFn"]},];
implementors["interledger_service_util"] = [{text:"impl&lt;S, O, A&gt; !Freeze for <a class=\"struct\" href=\"interledger_service_util/struct.BalanceService.html\" title=\"struct interledger_service_util::BalanceService\">BalanceService</a>&lt;S, O, A&gt;",synthetic:true,types:["interledger_service_util::balance_service::BalanceService"]},{text:"impl&lt;I, A&gt; !Freeze for <a class=\"struct\" href=\"interledger_service_util/struct.EchoService.html\" title=\"struct interledger_service_util::EchoService\">EchoService</a>&lt;I, A&gt;",synthetic:true,types:["interledger_service_util::echo_service::EchoService"]},{text:"impl&lt;S, O, A&gt; !Freeze for <a class=\"struct\" href=\"interledger_service_util/struct.ExchangeRateService.html\" title=\"struct interledger_service_util::ExchangeRateService\">ExchangeRateService</a>&lt;S, O, A&gt;",synthetic:true,types:["interledger_service_util::exchange_rates_service::ExchangeRateService"]},{text:"impl&lt;O&gt; Freeze for <a class=\"struct\" href=\"interledger_service_util/struct.ExpiryShortenerService.html\" title=\"struct interledger_service_util::ExpiryShortenerService\">ExpiryShortenerService</a>&lt;O&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;O: Freeze,&nbsp;</span>",synthetic:true,types:["interledger_service_util::expiry_shortener_service::ExpiryShortenerService"]},{text:"impl&lt;I&gt; !Freeze for <a class=\"struct\" href=\"interledger_service_util/struct.MaxPacketAmountService.html\" title=\"struct interledger_service_util::MaxPacketAmountService\">MaxPacketAmountService</a>&lt;I&gt;",synthetic:true,types:["interledger_service_util::max_packet_amount_service::MaxPacketAmountService"]},{text:"impl&lt;S, I, A&gt; !Freeze for <a class=\"struct\" href=\"interledger_service_util/struct.RateLimitService.html\" title=\"struct interledger_service_util::RateLimitService\">RateLimitService</a>&lt;S, I, A&gt;",synthetic:true,types:["interledger_service_util::rate_limit_service::RateLimitService"]},{text:"impl&lt;IO, A&gt; !Freeze for <a class=\"struct\" href=\"interledger_service_util/struct.ValidatorService.html\" title=\"struct interledger_service_util::ValidatorService\">ValidatorService</a>&lt;IO, A&gt;",synthetic:true,types:["interledger_service_util::validator_service::ValidatorService"]},{text:"impl Freeze for <a class=\"enum\" href=\"interledger_service_util/enum.RateLimitError.html\" title=\"enum interledger_service_util::RateLimitError\">RateLimitError</a>",synthetic:true,types:["interledger_service_util::rate_limit_service::RateLimitError"]},];
implementors["interledger_settlement"] = [{text:"impl&lt;S, O, A&gt; Freeze for <a class=\"struct\" href=\"interledger_settlement/struct.SettlementApi.html\" title=\"struct interledger_settlement::SettlementApi\">SettlementApi</a>&lt;S, O, A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;O: Freeze,<br>&nbsp;&nbsp;&nbsp;&nbsp;S: Freeze,&nbsp;</span>",synthetic:true,types:["interledger_settlement::api::SettlementApi"]},{text:"impl Freeze for <a class=\"struct\" href=\"interledger_settlement/struct.SettlementClient.html\" title=\"struct interledger_settlement::SettlementClient\">SettlementClient</a>",synthetic:true,types:["interledger_settlement::client::SettlementClient"]},{text:"impl&lt;I, A&gt; !Freeze for <a class=\"struct\" href=\"interledger_settlement/struct.SettlementMessageService.html\" title=\"struct interledger_settlement::SettlementMessageService\">SettlementMessageService</a>&lt;I, A&gt;",synthetic:true,types:["interledger_settlement::message_service::SettlementMessageService"]},{text:"impl Freeze for <a class=\"struct\" href=\"interledger_settlement/struct.SE_ILP_ADDRESS.html\" title=\"struct interledger_settlement::SE_ILP_ADDRESS\">SE_ILP_ADDRESS</a>",synthetic:true,types:["interledger_settlement::SE_ILP_ADDRESS"]},{text:"impl Freeze for <a class=\"struct\" href=\"interledger_settlement/struct.Quantity.html\" title=\"struct interledger_settlement::Quantity\">Quantity</a>",synthetic:true,types:["interledger_settlement::Quantity"]},{text:"impl Freeze for <a class=\"struct\" href=\"interledger_settlement/struct.SettlementEngineDetails.html\" title=\"struct interledger_settlement::SettlementEngineDetails\">SettlementEngineDetails</a>",synthetic:true,types:["interledger_settlement::SettlementEngineDetails"]},{text:"impl Freeze for <a class=\"struct\" href=\"interledger_settlement/struct.ConvertDetails.html\" title=\"struct interledger_settlement::ConvertDetails\">ConvertDetails</a>",synthetic:true,types:["interledger_settlement::ConvertDetails"]},];
implementors["interledger_spsp"] = [{text:"impl !Freeze for <a class=\"struct\" href=\"interledger_spsp/struct.SpspResponder.html\" title=\"struct interledger_spsp::SpspResponder\">SpspResponder</a>",synthetic:true,types:["interledger_spsp::server::SpspResponder"]},{text:"impl !Freeze for <a class=\"struct\" href=\"interledger_spsp/struct.SpspResponse.html\" title=\"struct interledger_spsp::SpspResponse\">SpspResponse</a>",synthetic:true,types:["interledger_spsp::SpspResponse"]},{text:"impl Freeze for <a class=\"enum\" href=\"interledger_spsp/enum.Error.html\" title=\"enum interledger_spsp::Error\">Error</a>",synthetic:true,types:["interledger_spsp::Error"]},];
implementors["interledger_stream"] = [{text:"impl !Freeze for <a class=\"struct\" href=\"interledger_stream/struct.ConnectionGenerator.html\" title=\"struct interledger_stream::ConnectionGenerator\">ConnectionGenerator</a>",synthetic:true,types:["interledger_stream::server::ConnectionGenerator"]},{text:"impl&lt;O, A&gt; !Freeze for <a class=\"struct\" href=\"interledger_stream/struct.StreamReceiverService.html\" title=\"struct interledger_stream::StreamReceiverService\">StreamReceiverService</a>&lt;O, A&gt;",synthetic:true,types:["interledger_stream::server::StreamReceiverService"]},{text:"impl Freeze for <a class=\"enum\" href=\"interledger_stream/enum.Error.html\" title=\"enum interledger_stream::Error\">Error</a>",synthetic:true,types:["interledger_stream::error::Error"]},];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
