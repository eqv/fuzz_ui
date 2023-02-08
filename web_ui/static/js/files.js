template_html = `
<ul class="file-list"> 
	<li v-for="(info,id) of file_infos" class=file_link> 
		<div class="file-line-link" @click="set_file(id)"> {{info.base_name}} ({{info.covered_lines}}/{{info.lines}}) </div>
	</li> 
</ul>
`

Vue.component('ijon-files', {
		template: template_html,
		props: ['file_infos', 'src_file'],
		methods: {
			set_file: function(file){
				this.$emit("select-line",file,1);
			}
		}
	})
