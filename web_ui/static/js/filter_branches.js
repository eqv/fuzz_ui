template_html = `
<ul class="file-list"> 
	<li v-for="line in filtered_lines" class=file_link> 
		<div v-if="line" class="file-line-link" @click="set_file(line)"> {{file_infos[line["file"]]["base_name"]}}:{{line["num"]}} </div>
	</li> 
</ul>
`

Vue.component('ijon-filter-branches', {
		template: template_html,
		props: ['file_infos', 'filtered_lines'],
		methods: {
			set_file: function(line_info){
				this.$emit("select-line",line_info["file"],line_info["num"]);
			}
		}
	})