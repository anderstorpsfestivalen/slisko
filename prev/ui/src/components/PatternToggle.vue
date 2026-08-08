<template>
	<div>
		<section>
			<b-field grouped>
				<b-switch
					v-model="enabled"
					@input="toggle(name, enabled)"
				></b-switch>
				<b>{{ name }}</b>
			</b-field>
		</section>
	</div>
</template>

<script>
export default {
	name: "PatternToggle",
	props: {
		name: {
			type: String,
			default: "",
		},
		enabled: {
			type: Boolean,
			default: false,
		},
	},
	mounted: function() {},
	methods: {
		toggle: function(pattern, status) {
			var hostname = "http://" + location.host + "/";

			if (window.webpackHotUpdate) {
				hostname = "http://" + window.location.hostname + ":3000/";
			}

			var action = "disable";
			if (status) {
				action = "enable";
			}

			fetch(hostname + "pattern/" + action + "/" + pattern);
		},
	},
};
</script>
