<template>
	<div class="container">
		<div class="row">
			<div class="col-sm">
				One of three columns
			</div>
			<div class="col-sm">
				One of three columns
			</div>
			<div class="col-sm">
				<PatternToggle :enabled="false"></PatternToggle>
			</div>
			{{ patterns.link }}
		</div>
	</div>
</template>

<script>
import PatternToggle from "./PatternToggle";
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
		this.$options.sockets.onmessage = (data) =>
			(this.patterns = JSON.parse(data.data));
	},
};
</script>
