(function() {var implementors = {};
implementors["interledger_ildcp"] = [{text:"impl&lt;I, A&gt; <a class=\"trait\" href=\"interledger_service/trait.IncomingService.html\" title=\"trait interledger_service::IncomingService\">IncomingService</a>&lt;A&gt; for <a class=\"struct\" href=\"interledger_ildcp/struct.IldcpService.html\" title=\"struct interledger_ildcp::IldcpService\">IldcpService</a>&lt;I, A&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;I: <a class=\"trait\" href=\"interledger_service/trait.IncomingService.html\" title=\"trait interledger_service::IncomingService\">IncomingService</a>&lt;A&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;A: <a class=\"trait\" href=\"interledger_ildcp/trait.IldcpAccount.html\" title=\"trait interledger_ildcp::IldcpAccount\">IldcpAccount</a>,&nbsp;</span>",synthetic:false,types:["interledger_ildcp::server::IldcpService"]},];
implementors["interledger_router"] = [{text:"impl&lt;S, O&gt; <a class=\"trait\" href=\"interledger_service/trait.IncomingService.html\" title=\"trait interledger_service::IncomingService\">IncomingService</a>&lt;&lt;S as <a class=\"trait\" href=\"interledger_service/trait.AccountStore.html\" title=\"trait interledger_service::AccountStore\">AccountStore</a>&gt;::<a class=\"type\" href=\"interledger_service/trait.AccountStore.html#associatedtype.Account\" title=\"type interledger_service::AccountStore::Account\">Account</a>&gt; for <a class=\"struct\" href=\"interledger_router/struct.Router.html\" title=\"struct interledger_router::Router\">Router</a>&lt;S, O&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"interledger_router/trait.RouterStore.html\" title=\"trait interledger_router::RouterStore\">RouterStore</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;O: <a class=\"trait\" href=\"interledger_service/trait.OutgoingService.html\" title=\"trait interledger_service::OutgoingService\">OutgoingService</a>&lt;S::<a class=\"type\" href=\"interledger_service/trait.AccountStore.html#associatedtype.Account\" title=\"type interledger_service::AccountStore::Account\">Account</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,&nbsp;</span>",synthetic:false,types:["interledger_router::router::Router"]},];
implementors["interledger_service"] = [];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
