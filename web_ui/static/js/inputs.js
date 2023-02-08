template_html = `
<ul class="input-list"> 
	<li v-for="inp in line_inputs"> 
		<a :href='"/static/queue.html#i="+inp'>{{input_infos[inp].path}}</a>
	</li> 
</ul>
`

Vue.component('ijon-inputs', {
		template: template_html,
		props: ['input_infos', 'line_inputs'],
		methods: {},
	})
