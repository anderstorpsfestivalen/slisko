<template>
	<div id="app">
		<div class="container">
			<section class="hero">
				<div class="hero-body">
					<p class="title">
						PENIS UI
					</p>
				</div>
			</section>
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
