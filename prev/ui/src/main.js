import Vue from "vue";
import App from "./App.vue";
import Buefy from "buefy";
import "buefy/dist/buefy.css";

Vue.use(Buefy);

Vue.config.productionTip = false;

var hostname = "ws://" + location.host + "/ws";

if (window.webpackHotUpdate) {
	hostname = "ws://" + window.location.hostname + ":3000/ws";
}

import VueNativeSock from "vue-native-websocket";
Vue.use(VueNativeSock, hostname, {
	format: "json",
	reconnection: true,
	reconnectionAttempts: 50,
	reconnectionDelay: 3000,
});

new Vue({
	render: (h) => h(App),
}).$mount("#app");
