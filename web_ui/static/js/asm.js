template_html = `
<div class="asmview">
	<pre v-if="asm">
<div 
		v-for="line in asm" 
			v-bind:id="'a'+line[0]" 
			v-bind:class="'line seta'"
			@click="set_addr(line[0])"
			:class='[{"selected": selected_addrs.includes(line[0])}, "line","seta"]'
	><span class="line_span" v-html="line[0]+'  '+line[1]"></span></div></pre>
</div>
`

Vue.component('ijon-asm-view', {
		template: template_html,
		props: ['asm', 'selected_addrs'],
		methods: {
			set_addr: function(addr){
                console.log("clicked on address",addr);
				//this.$emit("select-addr",addr);
			}
		}
	})
