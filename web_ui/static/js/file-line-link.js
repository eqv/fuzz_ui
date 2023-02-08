Vue.component('file-line-link', {
	props: ['file','line','curr_file', 'file_infos'],
	template: '<div class="file-line-link" @click="select_line()"> {{text}}</div>',
	methods:{
			select_line: function() {
				this.$emit("select-line",this.file,this.line);
			},
	},
	computed:{
		text: function(){
			if(this.file != this.curr_file){
				return this.file_infos[this.file].base_name+":"+this.line
			}else{
				return "line "+this.line
			}
		}
	}
})
