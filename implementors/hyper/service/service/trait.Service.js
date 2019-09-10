(function() {var implementors = {};
implementors["interledger_http"] = [{text:"impl&lt;S, T&gt; <a class=\"trait\" href=\"https://docs.rs/hyper/0.12.34/hyper/service/service/trait.Service.html\" title=\"trait hyper::service::service::Service\">Service</a> for <a class=\"struct\" href=\"interledger_http/struct.HttpServerService.html\" title=\"struct interledger_http::HttpServerService\">HttpServerService</a>&lt;S, T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"interledger_service/trait.IncomingService.html\" title=\"trait interledger_service::IncomingService\">IncomingService</a>&lt;T::<a class=\"type\" href=\"interledger_http/trait.HttpStore.html#associatedtype.Account\" title=\"type interledger_http::HttpStore::Account\">Account</a>&gt; + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"interledger_http/trait.HttpStore.html\" title=\"trait interledger_http::HttpStore\">HttpStore</a> + 'static,&nbsp;</span>",synthetic:false,types:["interledger_http::server::HttpServerService"]},];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
