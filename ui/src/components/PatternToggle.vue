<template>
	<div>
		<b-form-checkbox v-model="enabled" name="check-button" switch>
			Switch Checkbox <b>(Checked: {{ enabled }})</b>
		</b-form-checkbox>
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
};
</script>
