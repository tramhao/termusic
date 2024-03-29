//////////////////////////////////////////////////////////////////////////////
///
/// C# example that manipulates mp3 audio files with SoundTouch library.
/// 
/// Author        : Copyright (c) Olli Parviainen
/// Author e-mail : oparviai 'at' iki.fi
/// SoundTouch WWW: http://www.surina.net/soundtouch
///
////////////////////////////////////////////////////////////////////////////////
//
// License for this source code file: Microsoft Public License(Ms-PL)
//
////////////////////////////////////////////////////////////////////////////////

using soundtouch;
using System;
using System.IO;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;

namespace csharp_example
{
    /// <summary>
    /// Interaction logic for MainWindow.xaml
    /// </summary>
    public partial class MainWindow : Window
    {
        protected SoundProcessor processor = new SoundProcessor();

        public MainWindow()
        {
            InitializeComponent();

            StatusMessage.statusEvent += StatusEventHandler;
            processor.PlaybackStopped += EventHandler_playbackStopped;
            DisplaySoundTouchVersion();
        }


        /// <summary>
        /// Display SoundTouch library version string in status bar. This also indicates whether the DLL was loaded successfully or not ...
        /// </summary>
        private void DisplaySoundTouchVersion()
        {
            string status;
            try
            {
                status = String.Format("SoundTouch version: {0}", SoundTouch.Version);
            }
            catch (Exception exp)
            {
                status = exp.Message;
            }
            text_status.Text = status;
        }


        private void StatusEventHandler(object sender, string msg)
        {
            text_status.Text = msg;
        }


        // Open mp3 file for playback
        private void OpenFile(string fileName)
        {
            Stop();
            if (processor.OpenMp3File(fileName) == true)
            {
                textBox_filename.Text = fileName;
                button_play.IsEnabled = true;
                button_stop.IsEnabled = true;

                // Parse adjustment settings
                ParseTempoTextBox();
                ParsePitchTextBox();
                ParseRateTextBox();
            }
            else
            {
                textBox_filename.Text = "";
                button_play.IsEnabled = false;
                button_stop.IsEnabled = false;
                MessageBox.Show("Coudln't open audio file " + fileName);
            }
        }


        private void button_browse_Click(object sender, RoutedEventArgs e)
        {
            // Show file selection dialog
            Microsoft.Win32.OpenFileDialog openDialog = new Microsoft.Win32.OpenFileDialog();
            if (string.IsNullOrEmpty(textBox_filename.Text) == false)
            {
                // if an audio file is open, set directory to same as with the file
                openDialog.InitialDirectory = Path.GetDirectoryName(textBox_filename.Text);
            }
            openDialog.Filter = "MP3 files (*.mp3)|*.mp3";
            if (openDialog.ShowDialog() == true)
            {
                OpenFile(openDialog.FileName);
            }
        }


        private void setPlayButtonMode(bool play)
        {
            button_play.Content = play ? "_Play" : "_Pause";
        }


        private void EventHandler_playbackStopped(object sender, bool hasReachedEnd)
        {
            if (hasReachedEnd)
            {
                text_status.Text = "Stopped";
            }   // otherwise paused

            setPlayButtonMode(true);
        }


        private void button_play_Click(object sender, RoutedEventArgs e)
        {
            if ((string)button_play.Content == "_Pause")
            {
                // Pause
                if (processor.Pause())
                {
                    text_status.Text = "Paused";
                }
                setPlayButtonMode(true);
            }
            else
            {
                // Play
                if (processor.Play())
                {
                    text_status.Text = "Playing";
                    setPlayButtonMode(false);
                }
            }
        }


        private void Stop()
        {
            if (processor.Stop())
            {
                text_status.Text = "Stopped";
            }
            setPlayButtonMode(true);
        }


        private void button_stop_Click(object sender, RoutedEventArgs e)
        {
            Stop();
        }


        private bool parse_percentValue(TextBox box, out double value)
        {
            if (double.TryParse(box.Text, out value) == false) return false;
            if (value < -99.0) value = -99.0;   // don't allow more than -100% slowdown ... :)
            box.Text = value.ToString();
            return true;
        }


        private void ParsePitchTextBox()
        {
            double pitchValue;
            if (double.TryParse(textBox_pitch.Text, out pitchValue))
            {
                if (processor.streamProcessor != null) processor.streamProcessor.st.PitchSemiTones = (float)pitchValue;
            }
        }


        private void ParseTempoTextBox()
        {
            double tempoValue;
            if (parse_percentValue(textBox_tempo, out tempoValue))
            {
                if (processor.streamProcessor != null) processor.streamProcessor.st.TempoChange = (float)tempoValue;
            }
        }


        private void ParseRateTextBox()
        {
            double rateValue;
            if (parse_percentValue(textBox_rate, out rateValue))
            {
                if (processor.streamProcessor != null) processor.streamProcessor.st.RateChange = (float)rateValue;
            }
        }


        private void textBox_tempo_LostFocus(object sender, RoutedEventArgs e)
        {
            ParseTempoTextBox();
        }


        private void textBox_tempo_KeyDown(object sender, KeyEventArgs e)
        {
            if (e.Key == Key.Enter)
            {
                // enter pressed -- parse value
                ParseTempoTextBox();
            }
        }


        private void textBox_pitch_LostFocus(object sender, RoutedEventArgs e)
        {
            ParsePitchTextBox();
        }


        private void textBox_pitch_KeyDown(object sender, KeyEventArgs e)
        {
            if (e.Key == Key.Enter)
            {
                // enter pressed -- parse value
                ParsePitchTextBox();
            }
        }


        private void textBox_rate_LostFocus(object sender, RoutedEventArgs e)
        {
            ParseRateTextBox();
        }


        private void textBox_rate_KeyDown(object sender, KeyEventArgs e)
        {
            if (e.Key == Key.Enter)
            {
                // enter pressed -- parse value
                ParseRateTextBox();
            }
        }


        //  Handler for file drag & drop over the window
        private void Window_Drop(object sender, DragEventArgs e)
        {
            string[] files = (string[])e.Data.GetData(DataFormats.FileDrop);
            // open 1st of the chosen files
            OpenFile(files[0]);
        }
    }
}
