<template>
	<div class="container">
		<div class="columns">
			<div class="column">
				<h2>Ports</h2>
				<PatternToggle
					v-for="pattern in patterns.link"
					:key="pattern.PatternName"
					:enabled="pattern.Enabled"
					:name="pattern.PatternName"
				></PatternToggle>
			</div>
			<div class="column">
				<h2>Status</h2>
				<PatternToggle
					v-for="pattern in patterns.status"
					:key="pattern.PatternName"
					:enabled="pattern.Enabled"
					:name="pattern.PatternName"
				></PatternToggle>
			</div>
			<div class="column">
				<h2>Misc</h2>
				<PatternToggle
					v-for="pattern in patterns.misc"
					:key="pattern.PatternName"
					:enabled="pattern.Enabled"
					:name="pattern.PatternName"
				></PatternToggle>
			</div>
			<div class="column">
				<h2>Global</h2>
				<PatternToggle
					v-for="pattern in patterns.global"
					:key="pattern.PatternName"
					:enabled="pattern.Enabled"
					:name="pattern.PatternName"
				></PatternToggle>
			</div>
		</div>
	</div>
</template>

<script>
import PatternToggle from "./PatternToggle";
import _ from "lodash";
export default {
	name: "PatternList",
	props: {},
	data() {
		return {
			patterns: {},
		};
	},
	components: { PatternToggle },
	mounted: function() {},
	created: function() {
		this.$options.sockets.onmessage = (data) => {
			let m = JSON.parse(data.data);
			for (const [key, value] of Object.entries(m)) {
				m[key] = _.sortBy(value, ["PatternName"]);
			}
			this.patterns = m;
		};
	},
};
</script>
