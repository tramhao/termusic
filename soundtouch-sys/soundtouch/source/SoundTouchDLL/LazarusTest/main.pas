unit main;

{$mode objfpc}{$H+}

interface

uses
  Classes, SysUtils, Forms, Controls, Graphics, Dialogs, StdCtrls, SoundTouchDLL;


type

  { TForm1 }

  TForm1 = class(TForm)
    EditVersion: TEdit;
    Label1: TLabel;
    Load: TButton;

    procedure LoadClick(Sender: TObject);
  private

  public

  end;

var
  Form1: TForm1;

implementation

{$R *.lfm}

{ TForm1 }

procedure TForm1.LoadClick(Sender: TObject);
var
  version:string;
begin
  if IsSoundTouchLoaded() then
    version := SoundTouchGetVersionString()
  else
    version := '<library loading failed>';

  EditVersion.Text:= version;
end;

end.

