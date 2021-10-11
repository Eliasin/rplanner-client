export class QuillWrapper {
  constructor(selector) {
    this._quill = null;
  }

  spawn_quill(selector) {
    this._quill = new Quill(selector, {
      modules: {
        toolbar: [
            { 'header': 3 },
            { 'list': 'ordered' },
            {'list': 'bullet'},
            'code-block',
            'image',
            'bold', 'italic', 'strike'
        ]
      },
      placeholder: 'Compose an epic...',
      theme: 'snow'  // or 'bubble'
    });
  }

  get_content_from_index_and_length(index, length) {
    return JSON.stringify(this._quill.getContents(index, length));
  }

  get_content_from_index(index) {
    return JSON.stringify(this._quill.getContents(index));
  }

  get_content() {
    return JSON.stringify(this._quill.getContents());
  }
}
