template_html = `
<ul v-if="ts && ts[selected_line]" class=list-group>

	<li v-if="ts[selected_line].Taken" class=list-group-item> </i> Taken:
		<ul class="transitions-list"> 
			<li v-for="trans in ts[selected_line].Taken" :class='[trans.css]'> 
				<i :class='["fas",{"fa-sign-out-alt":!trans.is_xref, "fa-compress-arrows-alt":trans.is_xref}]'></i>  
				<file-line-link v-bind:file="trans.line.file" v-bind:line="trans.line.num" v-bind:curr_file="src_file" v-bind:file_infos="file_infos" @select-line="goto_line"></file-line-link>
			</li> 
		</ul>
	</li>

	<li v-if="ts[selected_line].NotTaken" class=list-group-item> </i> NotTaken:
		<ul class="transitions-list"> 
			<li v-for="trans in ts[selected_line].NotTaken" :class='[trans.css]'> 
				<i :class='["fas",{"fa-sign-out-alt":!trans.is_xref, "fa-compress-arrows-alt":trans.is_xref}]'></i>  
				<file-line-link v-bind:file="trans.line.file" v-bind:line="trans.line.num" v-bind:curr_file="src_file" v-bind:file_infos="file_infos" @select-line="goto_line"></file-line-link>
			</li> 
		</ul>
	</li>

	<li v-if="ts[selected_line].Direct" class=list-group-item> </i> Direct:
		<ul class="transitions-list"> 
			<li v-for="trans in ts[selected_line].Direct" :class='[trans.css]'> 
				<i :class='["fas",{"fa-sign-out-alt":!trans.is_xref, "fa-compress-arrows-alt":trans.is_xref}]'></i>  
				<file-line-link v-bind:file="trans.line.file" v-bind:line="trans.line.num" v-bind:curr_file="src_file" v-bind:file_infos="file_infos" @select-line="goto_line"></file-line-link>
			</li> 
		</ul>
	</li>

	<li v-if="ts[selected_line].Call" class=list-group-item> </i> Call:
		<ul class="transitions-list"> 
			<li v-for="trans in ts[selected_line].Call" :class='[trans.css]'> 
				<i :class='["fas",{"fa-sign-out-alt":!trans.is_xref, "fa-compress-arrows-alt":trans.is_xref}]'></i>  
				<file-line-link v-bind:file="trans.line.file" v-bind:line="trans.line.num" v-bind:curr_file="src_file" v-bind:file_infos="file_infos" @select-line="goto_line"></file-line-link>
			</li> 
		</ul>
	</li>

	<li v-if="ts[selected_line].Ret" class=list-group-item> </i> Ret:
		<ul class="transitions-list"> 
			<li v-for="trans in ts[selected_line].Ret" :class='[trans.css]'> 
				<i :class='["fas",{"fa-sign-out-alt":!trans.is_xref, "fa-compress-arrows-alt":trans.is_xref}]'></i>  
				<file-line-link v-bind:file="trans.line.file" v-bind:line="trans.line.num" v-bind:curr_file="src_file" v-bind:file_infos="file_infos" @select-line="goto_line"></file-line-link>
			</li> 
		</ul>
	</li>

	<li v-if="ts[selected_line].OutOfTrace" class=list-group-item> </i> Left Trace Region:
		<ul class="transitions-list"> 
			<li v-for="trans in ts[selected_line].OutOfTrace" :class='[trans.css]'> 
				<i :class='["fas",{"fa-sign-out-alt":!trans.is_xref, "fa-compress-arrows-alt":trans.is_xref}]'></i> 
				<file-line-link v-bind:file="trans.line.file" v-bind:line="trans.line.num" v-bind:curr_file="src_file" v-bind:file_infos="file_infos" @select-line="goto_line"></file-line-link>
			</li> 
		</ul>
	</li>
</ul>
`
Vue.component('ijon-line-transitions', {
		props: ['ts', 'src_file', 'selected_line', 'file_infos'],
		template: template_html,
		methods:{
			goto_line: function(file, line){
					this.$emit("select-line",file,line);
			}
		}
	})
