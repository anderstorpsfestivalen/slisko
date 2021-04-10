<template>
	<div id="app">
		<div class="container">
			<PatternList></PatternList>
		</div>
	</div>
</template>

<script>
import PatternList from "./components/PatternList";
export default {
	name: "App",
	components: { PatternList },
};
</script>

<style>
#app {
	font-family: Avenir, Helvetica, Arial, sans-serif;
	-webkit-font-smoothing: antialiased;
	-moz-osx-font-smoothing: grayscale;
	text-align: center;
	color: #2c3e50;
	margin-top: 60px;
}
</style>
