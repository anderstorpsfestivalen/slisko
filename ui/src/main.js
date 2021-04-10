import Vue from "vue";
import App from "./App.vue";
import { BootstrapVue, BootstrapVueIcons } from "bootstrap-vue";
import "bootstrap/dist/css/bootstrap.css";
import "bootstrap-vue/dist/bootstrap-vue.css";

Vue.config.productionTip = false;

var hostname = "ws://" + window.location.hostname + "/ws";

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

Vue.use(BootstrapVue);
Vue.use(BootstrapVueIcons);

new Vue({
	render: (h) => h(App),
}).$mount("#app");
