template_html = `
<div>
<input v-model="query" placeholder="main">
<input type="checkbox" id="only_cover" v-model="only_covered">
<label for="only_cover">Only Match Covered</label>
<ul class="file-list"> 
	<li v-for="line in filtered_lines" class=file_link> 
		<div class="file-line-link" @click="set_file(line)" > {{file_infos[line["file"]]["base_name"]}}:{{line["num"]}} </div>
	</li> 
</ul>
</div>
`

Vue.component('ijon-search', {
		template: template_html,
		data: function() {return {query: "", filtered_lines: [], only_covered: false} },
		props: ['file_infos'],
		methods: {
			set_file: function(line_info){
				this.$emit("select-line",line_info["file"],line_info["num"]);
			},
			search: function(){
				if(this.query.length > 3){
					var req = {pattern: this.query, only_covered: this.only_covered}
					this.$http.post('/api/search_pattern/',req).then( function (response) {
						this.$set(this, "filtered_lines", response.data);
					});
				}
			},
		},
		watch: {
			query: function(){ this.search(); },
			only_covered: function(){ this.search(); },
		}
	})
